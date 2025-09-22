use core::fmt;
use std::{
    cell::UnsafeCell,
    marker::PhantomData,
    ops::Deref,
    sync::atomic::{self, AtomicBool, AtomicPtr, AtomicUsize, Ordering},
};

use crate::{error::SendError, state::option::AtomicOption};

pub(super) struct Messages<T> {
    /// a ring buffer.
    ring: Vec<Seat<T>>,
    tail: AtomicUsize,
    closed: AtomicBool,
    len: usize,
}
#[derive(PartialEq, Eq)]
enum TailAction {
    DontUpdate,
    Update,
    ReadersAhead,
}

impl<T> Messages<T> {
    pub fn new(len: usize) -> Self {
        Self {
            ring: (0..len).map(|_| Seat::default()).collect(),
            tail: AtomicUsize::new(0),
            closed: AtomicBool::new(false),
            //todo: wondering why jon does this
            len,
        }
    }
    pub fn send(&self, value: T, num_readers: &AtomicUsize) -> Result<(), SendError<T>> {
        // this is my value
        let tail = self.tail.load(Ordering::SeqCst);

        // from Jon's notes in `bus`
        // we want to check if the next element over is free to ensure that we always leave one
        // empty space between the head and the tail. This is necessary so that readers can
        // distinguish between an empty and a full list. If the fence seat is free, the seat at
        // tail must also be free, which is simple enough to show by induction (exercise for the
        // reader).

        // Jon uses threading behaviors to implement this value, including
        // parking and referencing to the main thread.
        // I'm going to be more naive and not actually sleep threads.
        // Either it happens here, or it doesn't.
        // Jon uses an exponential spinwait
        // to avoid thread parking. see, I'm not gonna do that yet.

        // TODO: use Acquire :)

        // thoughts: so what if we have a fuck ton of broadcasters and not enough channels?
        // then we're going to have things write over each other.
        // We might want to assert that the number of broadcasters is less than the number of spaces.
        //
        // after that, we need to know if we can send the message.
        // the seat should have the information about whether it still is waiting to be broadcast.
        // We can do two things:
        // 1. allow that sender to overwrite the value in the buffer, and any remaining readers read
        // the next element? (and mind you, this probably is going to be totally fucked because maybe
        //  that next value is being written to as we speak, so the slowpoke would effectively
        // "jump ahead" and get data out of order)
        // 2. send an error.
        //
        // I'm going with #2 for now
        // lock on somewhere in the buffer

        //find a free space
        let mut i = 0;
        // update the t
        let mut update_tail = TailAction::DontUpdate;
        // we will loop through every place in the ring to find a free spot.
        let my_pos = loop {
            if i == self.len {
                let prev_pos = if i == 0 { 0 } else { (tail + i) % self.len };
                self.ring[prev_pos]
                    .check_writing
                    .store(false, Ordering::SeqCst);
                return Err(SendError::Full(value));
            }
            let try_pos = (tail + i + 1) % self.len;
            if self.ring[try_pos]
                .check_writing
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                i += 1;
                if update_tail == TailAction::DontUpdate {
                    update_tail = TailAction::Update;
                }
                continue;
            };

            // the number of readers may be less if a reader was dropped
            let required_reads = unsafe { (&*self.ring[try_pos].state.get()).required_reads }
                .min(num_readers.load(Ordering::Relaxed));
            // check if there are still readers here!
            if required_reads.saturating_sub(self.ring[try_pos].num_reads.load(Ordering::SeqCst))
                != 0
            {
                i += 1;
                update_tail = TailAction::ReadersAhead;
                continue;
            }

            break try_pos;
        };

        // i think? horsing around
        if update_tail == TailAction::Update {
            self.tail.fetch_add(i, Ordering::SeqCst);
        }
        // This is free to write!
        self.ring[my_pos].num_reads.store(0, Ordering::Release);
        let state = unsafe { &mut *self.ring[my_pos].state.get() };
        state.required_reads = num_readers.load(Ordering::Acquire);
        state.val = Some(value);
        Ok(())
    }
}

struct Seat<T> {
    // the number of reads
    num_reads: atomic::AtomicUsize,
    check_writing: AtomicBool,
    state: MutSeatState<T>,
}

impl<T> Default for Seat<T> {
    fn default() -> Self {
        // this is an empty seat. null pointer.
        // Nothing here.
        Self {
            num_reads: AtomicUsize::new(0),
            check_writing: AtomicBool::new(false),
            state: MutSeatState(UnsafeCell::new(SeatState {
                required_reads: 0,
                val: None,
            })),
        }
    }
}

struct MutSeatState<T>(UnsafeCell<SeatState<T>>);
impl<T> fmt::Debug for MutSeatState<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("MutSeatState").field(&self.0).finish()
    }
}
impl<T> Deref for MutSeatState<T> {
    type Target = UnsafeCell<SeatState<T>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct SeatState<T> {
    required_reads: usize,
    val: Option<T>,
}

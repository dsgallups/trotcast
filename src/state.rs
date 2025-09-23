use std::sync::atomic::{AtomicUsize, Ordering};

use crate::prelude::*;

/// TODO: we will need to
/// map dropped receivers to indices.
pub struct State<T> {
    /// a ring buffer.
    pub(crate) ring: Vec<Seat<T>>,
    /// in theory, this is used to point where the tail will be.
    pub(crate) tail: AtomicUsize,
    /// This keeps track of number of values to add to the writer_tail
    /// once the writer to writer_tail + 1 is complete
    pub(crate) num_writers: AtomicUsize,
    pub(crate) len: usize,
    /// keeps track of readers
    num_readers: AtomicUsize,
}

impl<T: Clone> State<T> {
    pub(crate) fn new(mut len: usize) -> Self {
        // needs one padding element
        len += 1;
        Self {
            ring: (0..len).map(|_| Seat::default()).collect(),
            tail: AtomicUsize::new(0),
            num_writers: AtomicUsize::new(0),
            //todo: wondering why jon does this
            len,
            num_readers: AtomicUsize::new(0),
        }
    }
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
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

        // Note: I tried having writers write to values later in the tail.
        // But this didn't work because you need a better sync implementation.
        let mut i = 0;
        let seat = loop {
            let tail = self.tail.load(Ordering::SeqCst);
            if self.ring[tail]
                .check_writing
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                i += 1;
                if i > 100 {
                    panic!("seems like ill loop forever. what happened?");
                }
                //yeah. SPIN LOCK.
                continue;
            };
            break tail;
        };

        let required_reads = unsafe { (&*self.ring[seat].state.get()).required_reads };

        if required_reads.saturating_sub(self.ring[seat].num_reads.load(Ordering::SeqCst)) != 0 {
            // release the check_write.
            self.ring[seat].check_writing.store(false, Ordering::SeqCst);
            return Err(SendError::Full(value));
        }

        // This is free to write!
        self.ring[seat].num_reads.store(0, Ordering::Release);
        let state = unsafe { &mut *self.ring[seat].state.get() };
        state.required_reads = self.num_readers.load(Ordering::SeqCst);
        state.val = Some(value);

        // set the tail last and then unlock check_writing
        let tail = seat + 1 % self.len;
        self.tail.store(tail, Ordering::Release);
        self.ring[seat].check_writing.store(false, Ordering::SeqCst);

        Ok(())
    }

    pub(crate) fn add_reader(&self) {
        self.num_readers.fetch_add(1, Ordering::Release);
    }

    pub(crate) fn add_writer(&self) {
        self.num_writers.fetch_add(1, Ordering::Release);
    }
}

// pub(crate) fn ring_id(val: usize, len: usize) -> usize {
//     (val + 1) % len
// }

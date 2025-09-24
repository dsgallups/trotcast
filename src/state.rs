use std::sync::{
    RwLock,
    atomic::{AtomicUsize, Ordering},
};

use crate::prelude::*;

#[derive(Default)]
struct Tail {
    position: usize,
}

pub struct State<T> {
    /// a ring buffer.
    pub(crate) ring: Vec<Seat<T>>,
    /// in theory, this is used to point where the tail will be.
    pub(crate) tail: AtomicUsize,
    /// This ensure that the state of `tail` is written to one at a time.
    internal_tail: RwLock<Tail>,
    /// This keeps track of number of values to add to the writer_tail
    /// once the writer to writer_tail + 1 is complete
    pub(crate) num_writers: AtomicUsize,
    pub(crate) len: usize,
    /// keeps track of readers
    pub(crate) num_readers: AtomicUsize,
}

impl<T: Clone> State<T> {
    pub(crate) fn new(mut len: usize) -> Self {
        // needs one padding element
        len += 1;
        Self {
            ring: (0..len).map(|_| Seat::default()).collect(),
            tail: AtomicUsize::new(0),
            internal_tail: RwLock::new(Tail::default()),
            num_writers: AtomicUsize::new(0),
            len,
            num_readers: AtomicUsize::new(0),
        }
    }
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        // Note: I tried having writers write to values later in the tail.
        // But this didn't work because you need a better sync implementation.

        // The element over has to be free to ensure there's always
        // an empty space between the head and the tail.

        if self.num_readers.load(Ordering::Relaxed) == 0 {
            return Err(SendError::Disconnected(value));
        }

        // I need sole access to the tail. other writers must wait on me.
        let mut tail_lock = self.internal_tail.write().unwrap();
        let fence = (tail_lock.position + 1) % self.len;

        let required_reads = unsafe { (&*self.ring[fence].state.get()).required_reads };

        // the fence has not yet been cleared of reads.
        if required_reads.saturating_sub(self.ring[fence].num_reads.load(Ordering::SeqCst)) != 0 {
            return Err(SendError::Full(value));
        }

        let seat = tail_lock.position;

        // This is free to write!
        let state = unsafe { &mut *self.ring[seat].state.get() };
        state.val = Some(value);

        state.required_reads = self.num_readers.load(Ordering::SeqCst);

        // set the tail last and then unlock check_writing
        let tail = (seat + 1) % self.len;
        self.ring[seat].num_writes.fetch_add(1, Ordering::Release);

        self.ring[seat].num_reads.store(0, Ordering::SeqCst);
        self.tail.store(tail, Ordering::SeqCst);
        tail_lock.position = tail;

        Ok(())
    }
}
/*
Ok I've found the problem
So for our reader, it reads 0. set the value to one and increments head to 1.

but the tail crashes into the reader's head.

the tail must always be ahead of all reader's heads.
if tail == reader's head, then the reader can't read again.

if the tail is moving faster than a reader's head

 */

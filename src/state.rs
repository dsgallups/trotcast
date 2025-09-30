use std::sync::{Mutex, atomic::AtomicUsize};

use crate::prelude::*;

/// Wrapper for the tail position in the ring buffer.
#[derive(Default)]
pub(crate) struct Tail(pub usize);

/// Core state of the broadcast channel managing the ring buffer and synchronization.
pub struct State<T> {
    /// a ring buffer.
    pub(crate) ring: Vec<Seat<T>>,
    /// in theory, this is used to point where the tail will be.
    pub(crate) tail: AtomicUsize,
    /// This ensure that the state of `tail` is written to one at a time.
    pub(crate) internal_tail: Mutex<Tail>,
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
            internal_tail: Mutex::new(Tail::default()),
            num_writers: AtomicUsize::new(0),
            len,
            num_readers: AtomicUsize::new(0),
        }
    }
}

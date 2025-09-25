use core::fmt;
use std::{
    cell::UnsafeCell,
    ops::Deref,
    sync::atomic::{AtomicUsize, Ordering},
};

/// A slot in the ring buffer that holds a value and tracks read operations.
pub(crate) struct Seat<T> {
    // the number of reads.
    // Readers never need to check if writing, because
    // writing will lock first, then determine if it's possible to
    // write based on number of reads.
    //
    // In the event a read and a write happen at the same time,
    // the channel will fail first
    pub(crate) num_reads: AtomicUsize,
    pub(crate) state: MutSeatState<T>,
}

impl<T> Default for Seat<T> {
    fn default() -> Self {
        Self {
            num_reads: AtomicUsize::new(0),
            state: MutSeatState(UnsafeCell::new(SeatState {
                required_reads: 0,
                val: None,
            })),
        }
    }
}

impl<T> fmt::Debug for Seat<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Seat")
            .field("num_reads", &self.num_reads)
            .field("state", &self.state)
            .finish()
    }
}

impl<T: Clone> Seat<T> {
    pub(crate) fn take(&self) -> T {
        let num_reads = self.num_reads.load(Ordering::SeqCst);

        let state = unsafe { &*self.state.get() };
        let required_reads = state.required_reads;
        assert!(
            num_reads < required_reads,
            "num_reads: {num_reads}, req: {}",
            required_reads
        );

        let value = if num_reads + 1 == state.required_reads {
            unsafe { &mut *self.state.get() }.val.take().unwrap()
        } else {
            state.val.clone().unwrap()
        };

        self.num_reads.fetch_add(1, Ordering::SeqCst);
        value
    }
}

pub(crate) struct MutSeatState<T>(UnsafeCell<SeatState<T>>);
impl<T> fmt::Debug for MutSeatState<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("MutSeatState")
            .field(&unsafe { &*self.0.get() }.required_reads)
            .finish()
    }
}
unsafe impl<T> Send for MutSeatState<T> {}
unsafe impl<T> Sync for MutSeatState<T> {}
impl<T> Deref for MutSeatState<T> {
    type Target = UnsafeCell<SeatState<T>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// State of a seat in the ring buffer containing the value and read requirements.
pub struct SeatState<T> {
    pub(crate) required_reads: usize,
    pub(crate) val: Option<T>,
}

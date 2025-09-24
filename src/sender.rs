use crate::prelude::*;
use std::sync::{Arc, atomic::Ordering};

/// A sender handle for the broadcast channel that allows sending messages to all receivers.
pub struct Sender<T> {
    shared: Arc<State<T>>,
}
impl<T: Clone> Sender<T> {
    pub(crate) fn new(shared: Arc<State<T>>) -> Self {
        shared.num_writers.fetch_add(1, Ordering::Release);
        Self { shared }
    }
    #[cfg(feature = "debug")]
    pub fn debugger(&self) -> Debug<T> {
        Debug {
            shared: Arc::clone(&self.shared),
        }
    }
    pub fn spawn_rx(&self) -> Receiver<T> {
        Receiver::new(Arc::clone(&self.shared))
    }

    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        if self.shared.num_readers.load(Ordering::Relaxed) == 0 {
            return Err(SendError::Disconnected(value));
        }

        // I need sole access to the tail. other writers must wait on me.
        let mut tail_lock = self.shared.internal_tail.write().unwrap();

        let fence = (tail_lock.0 + 1) % self.shared.len;

        let required_reads = unsafe { (&*self.shared.ring[fence].state.get()).required_reads };

        // the fence has not yet been cleared of reads.
        if required_reads.saturating_sub(self.shared.ring[fence].num_reads.load(Ordering::SeqCst))
            != 0
        {
            return Err(SendError::Full(value));
        }

        let seat = tail_lock.0;

        // This is free to write!
        let state = unsafe { &mut *self.shared.ring[seat].state.get() };
        state.val = Some(value);

        state.required_reads = self.shared.num_readers.load(Ordering::SeqCst);

        // set the tail last and then unlock check_writing
        let tail = (seat + 1) % self.shared.len;

        self.shared.ring[seat].num_reads.store(0, Ordering::SeqCst);
        self.shared.tail.store(tail, Ordering::SeqCst);
        tail_lock.0 = tail;

        Ok(())
    }
}

impl<T: Clone> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self::new(Arc::clone(&self.shared))
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.shared.num_writers.fetch_sub(1, Ordering::Release);
    }
}

use crate::prelude::*;
use std::sync::{Arc, atomic::Ordering};

/// A sender handle for the broadcast channel that allows sending messages to all receivers.
pub struct Channel<T> {
    shared: Arc<State<T>>,
}
impl<T: Clone> Channel<T> {
    /// Create a new channel
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity needs to be greater than 0");

        let shared = Arc::new(State::new(capacity));

        Self::from_shared_state(Arc::clone(&shared))
    }

    pub(crate) fn from_shared_state(shared: Arc<State<T>>) -> Self {
        shared.num_writers.fetch_add(1, Ordering::Release);
        Self { shared }
    }

    /// Spawns a debugger from the sender
    ///
    /// Enabled with the `debug` feature
    #[cfg(feature = "debug")]
    pub fn debugger(&self) -> Debug<T> {
        Debug {
            shared: Arc::clone(&self.shared),
        }
    }

    /// Spawns a new [`Receiver`]
    pub fn spawn_rx(&self) -> Receiver<T> {
        Receiver::new(Arc::clone(&self.shared))
    }

    fn send_inner(&self, value: T, blocking: bool) -> Result<(), SendError<T>> {
        if self.shared.num_readers.load(Ordering::Relaxed) == 0 {
            return Err(SendError::Disconnected(value));
        }

        loop {
            // I need sole access to the tail. other writers must wait on me.
            let mut tail_lock = self.shared.internal_tail.write().unwrap();

            let fence = (tail_lock.0 + 1) % self.shared.len;

            let required_reads = unsafe { (&*self.shared.ring[fence].state.get()).required_reads };

            // the fence has not yet been cleared of reads.
            if required_reads
                .saturating_sub(self.shared.ring[fence].num_reads.load(Ordering::SeqCst))
                != 0
            {
                if blocking {
                    continue;
                } else {
                    return Err(SendError::Full(value));
                }
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
            return Ok(());
        }
    }
    /// Sends a message. Will loop if the channel is full.
    ///
    /// # Errors
    /// - if there are no readers to receive the message.
    pub fn blocking_send(&self, value: T) -> Result<(), BlockingSendError<T>> {
        self.send_inner(value, true).map_err(|e| match e {
            SendError::Disconnected(val) => BlockingSendError::Disconnected(val),
            _ => unreachable!(),
        })
    }

    /// Sends a message.
    ///
    /// # Errors
    /// - if there are no readers to receive the message.
    /// - if the channel is full.
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        self.send_inner(value, false)
    }
}

impl<T: Clone> Clone for Channel<T> {
    fn clone(&self) -> Self {
        Self::from_shared_state(Arc::clone(&self.shared))
    }
}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        self.shared.num_writers.fetch_sub(1, Ordering::Release);
    }
}

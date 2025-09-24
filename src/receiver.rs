use crate::prelude::*;
use std::sync::{Arc, atomic::Ordering};

/// A receiver handle for the broadcast channel that allows for consuming messages.
///
/// ## Notes
///
/// If you have a receiver and don't read from it, that receiver
/// will block other receivers from receiving messages.
pub struct Receiver<T> {
    pub(crate) shared: Arc<State<T>>,
    pub(crate) closed: bool,
    #[cfg(feature = "debug")]
    pub head: usize,
    #[cfg(not(feature = "debug"))]
    pub(crate) head: usize,
}

impl<T: Clone> Receiver<T> {
    pub(crate) fn new(shared: Arc<State<T>>) -> Self {
        shared.num_readers.fetch_add(1, Ordering::Release);
        Self {
            closed: shared.num_writers.load(Ordering::Relaxed) == 0,
            head: shared.tail.load(Ordering::Relaxed),
            shared,
        }
    }
    /// Spawn a [`Sender`]
    pub fn spawn_tx(&self) -> Sender<T> {
        Sender::new(Arc::clone(&self.shared))
    }
    /// Try to receive a message.
    ///
    /// # Errors
    /// - if there's no new message available
    /// - if the channel is closed
    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        self.recv_inner(RecvCondition::Try).map_err(|e| match e {
            InnerRecvError::Disconnected => TryRecvError::Disconnected,
            InnerRecvError::Empty => TryRecvError::Empty,
        })
    }

    /// Receive a message. Loops until a message is available.
    ///
    /// # Errors
    /// - if the channel is closed
    pub fn recv(&mut self) -> Result<T, RecvError> {
        self.recv_inner(RecvCondition::Block).map_err(|e| match e {
            InnerRecvError::Disconnected => RecvError::Disconnected,
            _ => unreachable!(),
        })
    }
    fn recv_inner(&mut self, cond: RecvCondition) -> Result<T, InnerRecvError> {
        if self.closed {
            return Err(InnerRecvError::Disconnected);
        }
        let mut was_closed = false;
        loop {
            let tail = self.shared.tail.load(Ordering::Acquire);
            if tail != self.head {
                break;
            }
            if self.shared.num_writers.load(Ordering::Relaxed) == 0 {
                // from bus docs:
                //
                // we need to check again that there's nothing in the bus, otherwise we might have
                // missed a write between when we did the read of .tail above and when we read
                // .closed here
                if !was_closed {
                    was_closed = true;
                    continue;
                }
                self.closed = true;
                return Err(InnerRecvError::Disconnected);
            }
            if cond == RecvCondition::Try {
                return Err(InnerRecvError::Empty);
            }
        }

        let head = self.head;
        let ret = self.shared.ring[head].take();
        self.head = (head + 1) % self.shared.len;
        Ok(ret)
    }
}
#[derive(PartialEq, Eq)]
pub(crate) enum RecvCondition {
    Try,
    Block,
}

impl<T: Clone> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Receiver::new(Arc::clone(&self.shared))
    }
}
impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.shared.num_readers.fetch_sub(1, Ordering::Release);
        let mut cur = self.head;
        let tail = self.shared.tail.load(Ordering::SeqCst);
        // this probably means that some readers will lose info.
        while cur != tail {
            #[cfg(feature = "debug")]
            tracing::info!("Drop Proc: \nAdding 1 to {cur}");
            self.shared.ring[cur]
                .num_reads
                .fetch_add(1, Ordering::Release);
            cur = (cur + 1) % self.shared.len;
        }
    }
}

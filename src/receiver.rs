use crate::prelude::*;
use std::sync::{Arc, atomic::Ordering};

pub struct Receiver<T> {
    shared: Arc<State<T>>,
    closed: bool,
    pub head: usize,
}
impl<T> Receiver<T> {
    pub(crate) fn new(shared: Arc<State<T>>) -> Self {
        Self {
            shared,
            closed: false,
            head: 0,
        }
    }
}
impl<T: Clone> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        self.shared.add_reader();
        Self {
            shared: Arc::clone(&self.shared),
            closed: self.closed,
            head: self.head,
        }
    }
}

impl<T: Clone> Receiver<T> {
    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        self.recv_inner(RecvCondition::Try).map_err(|e| match e {
            InnerRecvError::Disconnected => TryRecvError::Disconnected,
            InnerRecvError::Empty => TryRecvError::Empty,
            InnerRecvError::Invalid => {
                panic!("invalidness");
            }
        })
    }
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

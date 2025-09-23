use crate::prelude::*;
use std::sync::Arc;

pub struct Receiver<T> {
    shared: Arc<State<T>>,
    closed: bool,
}
impl<T> Receiver<T> {
    pub(crate) fn new(shared: Arc<State<T>>) -> Self {
        Self {
            shared,
            closed: false,
        }
    }
}
impl<T: Clone> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        self.shared.add_reader();
        Self {
            shared: Arc::clone(&self.shared),
            closed: self.closed,
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
        let pos = self.shared.read_next(self.id);

        todo!()
    }
}

pub(crate) enum RecvCondition {
    Try,
    Block,
}

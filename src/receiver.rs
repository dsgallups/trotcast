use crate::prelude::*;
use std::{sync::Arc, time};

pub struct Receiver<T> {
    shared: Arc<State<T>>,
    id: usize,
    closed: bool,
}
impl<T> Receiver<T> {
    pub(crate) fn new(shared: Arc<State<T>>) -> Self {
        Self {
            shared,
            id: 0,
            closed: false,
        }
    }
}
impl<T: Clone> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        todo!()
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
        let pos = self.shared.read_next(self.id, cond);

        todo!()
    }
}

pub(crate) enum RecvCondition {
    Try,
    Block,
}

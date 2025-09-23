use crate::prelude::*;
use std::sync::Arc;

pub struct Receiver<T> {
    shared: Arc<State<T>>,
    closed: bool,
    pos: usize,
}
impl<T> Receiver<T> {
    pub(crate) fn new(shared: Arc<State<T>>) -> Self {
        Self {
            shared,
            closed: false,
            pos: 0,
        }
    }
}
impl<T: Clone> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        self.shared.add_reader();
        Self {
            shared: Arc::clone(&self.shared),
            closed: self.closed,
            pos: self.pos,
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
        loop {
            match self.shared.read(self.pos) {
                Ok(Some(value)) => {
                    return Ok(value);
                }
                Ok(None) => {
                    return Err(InnerRecvError::Empty);
                }
                Err(msg) => match msg {
                    MessageReadErr::BusyWriting => {
                        if cond == RecvCondition::Block {
                            continue;
                        } else {
                            return Err(InnerRecvError::Empty);
                        }
                    }
                    MessageReadErr::InvalidReader => {
                        panic!("Invalid reader found!");
                    }
                },
            }
        }
    }
}
#[derive(PartialEq, Eq)]
pub(crate) enum RecvCondition {
    Try,
    Block,
}

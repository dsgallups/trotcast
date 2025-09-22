use crate::prelude::*;
use std::sync::Arc;

pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
    id: usize,
}
impl<T> Receiver<T> {
    pub(crate) fn new(shared: Arc<Shared<T>>) -> Self {
        Self { shared, id: 0 }
    }
}
impl<T: Clone> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl<T: Clone> Receiver<T> {
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        todo!()
    }
    pub fn recv(&self) -> Result<T, RecvError> {
        todo!()
    }
}

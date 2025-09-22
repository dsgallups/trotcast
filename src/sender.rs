use crate::prelude::*;
use std::sync::Arc;

pub struct Sender<T> {
    shared: Arc<State<T>>,
}
impl<T> Sender<T> {
    pub(crate) fn new(shared: Arc<State<T>>) -> Self {
        Self { shared }
    }
}

impl<T: Clone> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T: Clone> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        self.shared.send(value)
    }
}

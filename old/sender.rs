use crate::prelude::*;
use std::sync::Arc;

pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}
impl<T> Sender<T> {
    pub(crate) fn new(shared: Arc<Shared<T>>) -> Self {
        Self { shared }
    }
}

impl<T: Clone> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        {
            let recv = self.shared.receiver_positions.read().unwrap();
            if recv.is_empty() {
                return Err(SendError::Disconnected(value));
            }
        }

        //TODO: don't unwrap
        //let mut state = self.shared.state.lock().unwrap();

        todo!()
    }
}

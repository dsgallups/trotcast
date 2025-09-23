use crate::prelude::*;
use std::sync::{Arc, atomic::Ordering};

pub struct Sender<T> {
    id: usize,
    shared: Arc<State<T>>,
}
impl<T: Clone> Sender<T> {
    pub(crate) fn new(shared: Arc<State<T>>) -> Self {
        shared.add_writer();
        Self { id: 0, shared }
    }
    pub fn debugger(&self) -> Debug<T> {
        Debug {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T: Clone> Clone for Sender<T> {
    fn clone(&self) -> Self {
        self.shared.add_writer();
        Self {
            id: self.id + 1,
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T: Clone> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        self.shared.send(value)
    }
}
impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.shared.num_writers.fetch_sub(1, Ordering::Release);
    }
}

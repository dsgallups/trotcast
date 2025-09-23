use std::sync::{Arc, atomic::Ordering};

use crate::prelude::*;

pub struct Spawner<T> {
    pub(crate) shared: Arc<State<T>>,
}

impl<T: Clone> Spawner<T> {
    pub fn spawn_rx(&self) -> Receiver<T> {
        self.shared.add_reader();
        Receiver {
            shared: Arc::clone(&self.shared),
            closed: self.shared.num_writers.load(Ordering::Relaxed) == 0,
            head: self.shared.tail.load(Ordering::Relaxed),
        }
    }
}

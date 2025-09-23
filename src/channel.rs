use std::sync::Arc;

use crate::prelude::*;

pub fn channel<T: Clone>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    assert!(capacity > 0, "Capacity needs to be greater than 0");

    let shared = Arc::new(State::new(capacity));

    shared.add_reader();
    shared.add_writer();

    let sender = Sender::new(Arc::clone(&shared));

    let receiver = Receiver::new(shared);

    (sender, receiver)
}

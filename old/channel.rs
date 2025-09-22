use std::sync::Arc;

use crate::prelude::*;

pub fn channel<T: Clone>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    assert!(capacity > 0, "Capacity needs to be greater than 0");

    let shared = Arc::new(Shared::new(capacity));

    let sender = Sender::new(Arc::clone(&shared));

    let receiver = Receiver::new(shared);

    (sender, receiver)
}

#![doc = r#"
A multi-producer, multi-consumer broadcast channel implementation.

This crate provides a broadcast channel where multiple senders can send messages
and multiple receivers will each receive a copy of every message sent.

# Example

```
// Create a broadcast channel with a capacity of 2
let (tx, mut rx1) = trotcast::channel(2);

// Clone the sender and receiver for multiple producers/consumers
let tx2 = tx.clone();
let mut rx2 = rx1.clone();

std::thread::spawn(move || {
    tx.blocking_send(1).unwrap();
    tx.blocking_send(2).unwrap();
});

std::thread::spawn(move || {
    tx2.blocking_send(3).unwrap();
    tx2.blocking_send(4).unwrap();
});

// Using crossbeam to validate the example
let (tx_vals, receiver_vals) = crossbeam_channel::unbounded();

let recv_1 = std::thread::spawn({
    let tx = tx_vals.clone();
    move || {
        let mut count = 0;
        // Collect messages from first receiver
        while let Ok(msg) = rx1.recv() {
            tx.send((1, msg)).unwrap();
            count += 1;
            if count == 4 {
                break;
            }
        }
    }
});

let recv_2 = std::thread::spawn({
    let tx = tx_vals.clone();
    move || {
        let mut count = 0;
        // Collect messages from first receiver
        while let Ok(msg) = rx2.recv() {
            tx.send((2, msg)).unwrap();
            count += 1;
            if count == 4 {
                break;
            }
        }
    }
});
loop {
    if [&recv_1, &recv_2].iter().all(|handle| handle.is_finished()) {
        break;
    }
}
let mut messages1 = vec![];
let mut messages2 = vec![];
while let Ok((thread_no, val)) = receiver_vals.try_recv() {
    if thread_no == 1 {
        messages1.push(val)
    } else {
        messages2.push(val)
    }
}
// Both receivers got all messages
assert_eq!(messages1.len(), 4);
assert_eq!(messages2.len(), 4);

for i in [1, 2, 3, 4] {
    assert!(messages1.contains(&i));
    assert!(messages2.contains(&i));
}
```

"#]

use std::sync::Arc;

use crate::prelude::*;

pub mod error;
pub mod receiver;
pub mod seat;
pub mod sender;
pub mod state;

#[cfg(feature = "debug")]
pub mod debug;

pub fn channel<T: Clone>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    assert!(capacity > 0, "Capacity needs to be greater than 0");

    let shared = Arc::new(State::new(capacity));

    let sender = Sender::new(Arc::clone(&shared));
    let receiver = Receiver::new(shared);

    (sender, receiver)
}

pub mod prelude {
    pub use crate::channel;
    pub use crate::error::*;
    pub use crate::receiver::*;
    pub(crate) use crate::seat::*;
    pub use crate::sender::*;
    pub use crate::state::*;

    #[cfg(feature = "debug")]
    pub use crate::debug::*;
}

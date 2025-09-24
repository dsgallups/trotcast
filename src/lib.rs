#![doc = r#"
A multi-producer, multi-consumer broadcast channel implementation.

This crate provides a broadcast channel where multiple senders can send messages
and multiple receivers will each receive a copy of every message sent.

# Overview

There are three structures to handle message passing

## [`trotcast::channel`](channel)
Creates a new channel with a single sender and receiver.

## [`Sender`]

A sender handle for the broadcast channel that allows sending messages to all receivers.

You can clone senders. If you need another `Receiver`, you can call `Receiver::spawn_rx`.

Senders will lock a `RwLock` when writing. I tried to not do this. If this behavior is undesirable, and you are aware of a better solution, please let me know!


## [`Receiver`]
A receiver handle for the broadcast channel that allows consuming messages.

Note: a channel should always have one active receiver. If you do not read
from all receivers, then your channel will be blocked on the non-reading receiver.

However, other receivers will be able to receiver prior messages until reaching
the state of the non-reading receiver.


You can clone receivers. If you need another `Sender`, you can call `Receiver::spawn_tx`.

Recievers will not lock any `Mutex` or `RwLock`.

# Example

```
use trotcast::prelude::*;
// Create a broadcast channel with a capacity of 2
let tx = Channel::new(2);

// Clone the sender, and create receivers for multiple producers/consumers
let tx2 = tx.clone();
let mut rx1 = tx.spawn_rx();
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

/// Error types
pub mod error;

mod receiver;
pub use receiver::*;

mod channel;
pub use channel::*;

pub(crate) mod seat;

pub(crate) mod state;

#[cfg(feature = "debug")]
pub mod debug;

pub mod prelude {
    pub use crate::channel::*;
    pub use crate::error::*;
    pub use crate::receiver::*;
    pub(crate) use crate::seat::*;
    pub(crate) use crate::state::*;

    #[cfg(feature = "debug")]
    pub use crate::debug::*;
}

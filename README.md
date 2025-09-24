# `Trotcast`

A parked crate for an `mpmc` broadcast impl



## Structures

### `Sender<T>`

A sender handle for the broadcast channel that allows sending messages to all receivers.

You can clone senders. If you need another `Receiver`, you can call `Receiver::spawn_rx`.

---

### `Receiver<T>`
A receiver handle for the broadcast channel that allows consuming messages.

Note: a channel should always have one active receiver. If you do not read
from all receivers, then your channel will be blocked on the non-reading receiver.

However, other receivers will be able to receiver prior messages until reaching
the state of the non-reading receiver.


You can clone receivers. If you need another `Sender`, you can call `Receiver::spawn_tx`.

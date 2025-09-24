# `Trotcast`

a `mpmc` broadcast impl.

See the docs for more information.



## Structures

### `Sender<T>`

A sender handle for the broadcast channel that allows sending messages to all receivers.

You can clone senders. If you need another `Receiver`, you can call `Receiver::spawn_rx`.

Senders will lock a `RwLock` when writing. I tried to not do this. If this behavior is undesirable, and you are aware of a better solution, please let me know!

---

### `Receiver<T>`
A receiver handle for the broadcast channel that allows consuming messages.

Note: a channel should always have one active receiver. If you do not read
from all receivers, then your channel will be blocked on the non-reading receiver.

However, other receivers will be able to receiver prior messages until reaching
the state of the non-reading receiver.


You can clone receivers. If you need another `Sender`, you can call `Receiver::spawn_tx`.

Recievers will not lock any `Mutex` or `RwLock`.

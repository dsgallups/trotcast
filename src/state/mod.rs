mod inner;
mod message;
mod option;

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{
    prelude::*,
    state::message::{MessageReadErr, Messages},
};

/// TODO: we will need to
/// map dropped receivers to indices.
pub struct State<T> {
    messages: Messages<T>,
    num_readers: AtomicUsize,
}

impl<T: Clone> State<T> {
    pub(crate) fn new(mut len: usize) -> Self {
        // needs one padding element
        len += 1;
        Self {
            messages: Messages::new(len),
            num_readers: AtomicUsize::new(0),
        }
    }
    pub(crate) fn send(&self, value: T) -> Result<(), SendError<T>> {
        self.messages.send(value, &self.num_readers)
    }

    pub(crate) fn add_reader(&self) {
        self.num_readers.fetch_add(1, Ordering::SeqCst);
    }
    pub(crate) fn read(&self, pos: usize) -> Result<Option<T>, MessageReadErr> {
        todo!()
    }

    // pub(crate) fn read_next(&self, id: usize) -> Result<Option<T>, MessageReadErr> {
    //     let tail = self.inner.get_tail(id)?;
    //     let res = self.messages.read(tail)?;
    //     self.inner.increment_tail(id)?;
    //     Ok(res)
    // }
    // pub(crate) fn add_recv(self: &Arc<Self>) -> (usize, Arc<Self>) {
    //     let id = self.next_receiver_id.fetch_add(1, Ordering::SeqCst);
    //     let id = self.inner.add_reader(id);

    //     (id, Arc::clone(self))
    // }
}

fn ring_id(val: usize, len: usize) -> usize {
    (val + 1) % len
}

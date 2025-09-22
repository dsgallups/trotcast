mod inner;
mod message;
mod option;

use std::sync::atomic::AtomicUsize;

use crate::{
    prelude::*,
    state::{
        inner::StateInner,
        message::{MessageReadErr, Messages},
    },
};

/// TODO: we will need to
/// map dropped receivers to indices.
pub struct State<T> {
    messages: Messages<T>,
    inner: StateInner,
    //positions: ReceiverPositions,
    next_receiver_id: AtomicUsize,
}

impl<T: Clone> State<T> {
    pub(crate) fn new(mut len: usize) -> Self {
        // needs one padding element
        len += 1;
        Self {
            messages: Messages::new(len),
            inner: StateInner::default(),
            next_receiver_id: AtomicUsize::new(0),
        }
    }
    pub(crate) fn send(&self, value: T) -> Result<(), SendError<T>> {
        self.messages.send(value, &self.inner)
    }
    pub(crate) fn read_next(&self, id: usize) -> Result<Option<T>, MessageReadErr> {
        let tail = self.inner.get_tail(id)?;
        let res = self.messages.read(tail)?;
        self.inner.increment_tail(id)?;
        Ok(res)
    }
}

fn ring_id(val: usize, len: usize) -> usize {
    (val + 1) % len
}

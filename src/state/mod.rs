mod message;
mod option;

use std::sync::atomic::AtomicUsize;

use crate::{prelude::*, state::message::Messages};

/// TODO: we will need to
/// map dropped receivers to indices.
pub struct State<T> {
    state: Messages<T>,
    num_readers: AtomicUsize,
    //positions: ReceiverPositions,
    next_receiver_id: AtomicUsize,
}

impl<T> State<T> {
    pub(crate) fn new(mut len: usize) -> Self {
        // needs one padding element
        len += 1;
        Self {
            state: Messages::new(len),
            num_readers: AtomicUsize::new(0),
            next_receiver_id: AtomicUsize::new(0),
        }
    }
    pub(crate) fn send(&self, value: T) -> Result<(), SendError<T>> {
        self.state.send(value, &self.num_readers)
    }
}

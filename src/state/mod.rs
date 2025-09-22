mod message;
mod option;

use std::sync::atomic::AtomicUsize;

use crate::{prelude::*, state::message::Messages};

/// TODO: we will need to
/// map dropped receivers to indices.
pub struct State<T> {
    state: Messages<T>,
    //positions: ReceiverPositions,
    next_receiver_id: AtomicUsize,
}

impl<T> State<T> {
    pub(crate) fn new(mut len: usize) -> Self {
        // needs one padding element
        len += 1;
        Self {
            state: Messages::new(len),
            next_receiver_id: AtomicUsize::new(0),
        }
    }
    pub(crate) fn send(&self, value: T) -> Result<(), SendError<T>> {
        {
            let mut writer = self.state.write().unwrap();
            writer.send(value)
        }
        todo!()
    }
}

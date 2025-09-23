use std::sync::atomic::{AtomicUsize, Ordering};

use crate::state::message::MessageReadErr;

// #[derive(Default)]
// pub struct StateInner {
//     readers: Vec<ReaderState>,
//     num_readers: usize,
// }

// impl StateInner {
//     pub fn get_tail(&self, id: usize) -> Result<usize, MessageReadErr> {
//         let state = self.readers.get(&id).ok_or(MessageReadErr::InvalidReader)?;
//         Ok(state.pos.load(Ordering::Relaxed))
//     }
//     pub fn increment_tail(&self, id: usize) -> Result<(), MessageReadErr> {
//         let state = self
//             .readers
//             .read()
//             .unwrap()
//             .get(&id)
//             .ok_or(MessageReadErr::InvalidReader)?;
//         state.pos.fetch_add(1, Ordering::Relaxed);
//         Ok(())
//     }
//     pub fn num_readers(&self) -> usize {
//         self.num_readers
//     }
//     pub fn add_reader(&self, id: usize) -> usize {
//         self.readers.lock
//     }
// }

// pub struct ReaderState {
//     pos: AtomicUsize,
// }

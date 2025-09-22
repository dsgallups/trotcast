use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::error::InnerRecvError;

#[derive(Default)]
pub struct StateInner {
    readers: HashMap<usize, ReaderState>,
    num_readers: usize,
}

impl StateInner {
    pub fn get_tail(&self, id: usize) -> Result<usize, InnerRecvError> {
        let state = self.readers.get(&id).ok_or(InnerRecvError::Invalid)?;
        Ok(state.pos.load(Ordering::Relaxed))
    }
    pub fn increment_tail(&self, id: usize) -> Result<(), InnerRecvError> {
        let state = self.readers.get(&id).ok_or(InnerRecvError::Invalid)?;
        state.pos.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    pub fn num_readers(&self) -> usize {
        self.num_readers
    }
}

pub struct ReaderState {
    pos: AtomicUsize,
}

use std::{
    collections::HashMap,
    sync::{Condvar, RwLock, atomic::AtomicUsize},
};

use crate::prelude::*;

pub struct Shared<T> {
    pub(crate) state: RwLock<State<T>>,
    pub(crate) positions: ReceiverPositions,
    pub(crate) next_receiver_id: AtomicUsize,
}

impl<T> Shared<T> {
    pub(crate) fn new(capacity: usize) -> Self {
        let state = RwLock::new(State {
            buf: Vec::with_capacity(capacity),
            capacity,
            write_pos: 0,
            num_senders: 1,
            closed: false,
        });
        Self {
            state,
            positions: ReceiverPositions::default(),
            next_receiver_id: AtomicUsize::new(1),
        }
    }
}

struct ReceiverPositions(RwLock<HashMap<usize, AtomicUsize>>);
impl Default for ReceiverPositions {
    fn default() -> Self {
        Self(RwLock::new(HashMap::new()))
    }
}

pub struct State<T> {
    pub(crate) buf: Vec<T>,
    pub(crate) capacity: usize,
    pub(crate) write_pos: usize,
    pub(crate) num_senders: usize,
    pub(crate) closed: bool,
}

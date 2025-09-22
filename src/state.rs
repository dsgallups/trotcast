use std::{
    collections::HashMap,
    sync::{Condvar, Mutex, atomic::AtomicUsize},
};

use crate::prelude::*;

pub struct Shared<T> {
    state: Mutex<State<T>>,
    condvar: Condvar,
    next_receiver_id: AtomicUsize,
}

impl<T> Shared<T> {
    pub(crate) fn new(capacity: usize) -> Self {
        let state = Mutex::new(State {
            buf: Vec::with_capacity(capacity),
            capacity,
            write_pos: 0,
            receiver_positions: HashMap::new(),
            num_senders: 1,
            closed: false,
        });
        Self {
            state,
            condvar: Condvar::new(),
            next_receiver_id: AtomicUsize::new(1),
        }
    }
}

pub struct State<T> {
    buf: Vec<T>,
    capacity: usize,
    write_pos: usize,
    receiver_positions: HashMap<usize, usize>,
    num_senders: usize,
    closed: bool,
}

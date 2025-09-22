use std::{
    collections::HashMap,
    marker::PhantomData,
    sync::{Condvar, RwLock, atomic::AtomicUsize},
};

use crate::prelude::*;

pub struct Shared<T> {
    //state: RwLock<State<T>>,
    //positions: ReceiverPositions,
    next_receiver_id: AtomicUsize,
    _val: PhantomData<T>,
}

impl<T> Shared<T> {
    pub(crate) fn new(capacity: usize) -> Self {
        todo!()
    }
}

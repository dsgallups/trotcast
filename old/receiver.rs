use crate::prelude::*;
use std::sync::Arc;

pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
    id: usize,
}
impl<T> Receiver<T> {
    pub(crate) fn new(shared: Arc<Shared<T>>) -> Self {
        Self { shared, id: 0 }
    }
}

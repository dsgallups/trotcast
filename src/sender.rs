use crate::prelude::*;
use std::sync::Arc;

pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}
impl<T> Sender<T> {
    pub(crate) fn new(shared: Arc<Shared<T>>) -> Self {
        Self { shared }
    }
}

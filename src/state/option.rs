use core::fmt;
use std::{marker::PhantomData, ptr, sync::atomic::AtomicPtr};

pub struct AtomicOption<T> {
    ptr: AtomicPtr<T>,
    _marker: PhantomData<Option<Box<T>>>,
}

impl<T> AtomicOption<T> {
    pub fn empty() -> Self {
        Self {
            ptr: AtomicPtr::new(ptr::null_mut()),
            _marker: PhantomData,
        }
    }
}

impl<T> fmt::Debug for AtomicOption<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AtomicOption")
            .field("ptr", &self.ptr)
            .finish()
    }
}

unsafe impl<T: Send> Send for AtomicOption<T> {}
unsafe impl<T: Send> Sync for AtomicOption<T> {}

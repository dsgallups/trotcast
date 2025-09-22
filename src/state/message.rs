use core::fmt;
use std::{
    cell::UnsafeCell,
    marker::PhantomData,
    sync::atomic::{self, AtomicBool, AtomicPtr, AtomicUsize, Ordering},
};

use crate::state::option::AtomicOption;

pub(super) struct Messages<T> {
    /// a ring buffer.
    ring: Vec<Seat<T>>,
    tail: AtomicUsize,
    closed: AtomicBool,
    len: usize,
}

impl<T> Messages<T> {
    pub fn new(len: usize) -> Self {
        Self {
            ring: (0..len).map(|_| Seat::default()).collect(),
            tail: AtomicUsize::new(0),
            closed: AtomicBool::new(false),
            len,
        }
    }
    pub fn send(&mut self, value: T) {
        // TODO: make sure we use a Load when setting this value
        let tail = self.tail.load(Ordering::Acquire);

        // we want to check if the next element over is free to ensure that we always leave one
        // empty space between the head and the tail. This is necessary so that readers can
        // distinguish between an empty and a full list. If the fence seat is free, the seat at
        // tail must also be free, which is simple enough to show by induction (exercise for the
        // reader).
        let fence = (tail + 1) % self.state.len;
    }
}

struct Seat<T> {
    read: atomic::AtomicUsize,
    waiting: AtomicOption<T>,
    state: MutSeatState<T>,
}

impl<T> Default for Seat<T> {
    fn default() -> Self {
        // this is an empty seat. null pointer.
        // Nothing here.
        Self {
            read: AtomicUsize::new(0),
            waiting: AtomicOption::empty(),
            state: MutSeatState(UnsafeCell::new(SeatState { max: 0, val: None })),
        }
    }
}

struct MutSeatState<T>(UnsafeCell<SeatState<T>>);
impl<T> fmt::Debug for MutSeatState<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("MutSeatState").field(&self.0).finish()
    }
}

struct SeatState<T> {
    max: usize,
    val: Option<T>,
}

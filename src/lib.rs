//! A multi-producer, multi-consumer broadcast channel implementation.
//!
//! This crate provides a broadcast channel where multiple senders can send messages
//! and multiple receivers will each receive a copy of every message sent.
//!
//! TODO
pub mod channel;
pub mod error;
pub mod receiver;
pub mod seat;
pub mod sender;
pub mod state;

#[cfg(feature = "debug")]
pub mod debug;

pub mod prelude {
    pub use crate::channel::*;
    pub use crate::error::*;
    pub use crate::receiver::*;
    pub(crate) use crate::seat::*;
    pub use crate::sender::*;
    pub use crate::state::*;

    #[cfg(feature = "debug")]
    pub use crate::debug::*;
}

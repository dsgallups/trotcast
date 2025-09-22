//! A multi-producer, multi-consumer broadcast channel implementation.
//!
//! This crate provides a broadcast channel where multiple senders can send messages
//! and multiple receivers will each receive a copy of every message sent.
//!
//! TODO
pub mod channel;
pub mod error;
pub mod receiver;
pub mod sender;
pub mod state;

pub mod prelude {
    pub use crate::channel::*;
    pub use crate::error::*;
    pub use crate::receiver::*;
    pub use crate::sender::*;
    pub use crate::state::*;
}

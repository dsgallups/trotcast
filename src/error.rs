use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq)]
pub enum TryRecvError {
    Empty,
    Disconnected,
}
impl fmt::Display for TryRecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TryRecvError::Disconnected => write!(f, "Channel Disconnected"),
            TryRecvError::Empty => write!(f, "Channel Empty"),
        }
    }
}

impl Error for TryRecvError {}

#[derive(Debug, Clone, PartialEq)]
pub enum RecvError {
    Disconnected,
}

impl fmt::Display for RecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecvError::Disconnected => write!(f, "Channel Disconnected"),
        }
    }
}
impl Error for RecvError {}

#[derive(Clone, PartialEq)]
pub enum SendError<T> {
    Disconnected(T),
    Full(T),
}

impl<T> fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SendError::Disconnected(_) => write!(f, "Channel Disconnected"),
            SendError::Full(_) => write!(f, "Channel Full"),
        }
    }
}
impl<T> fmt::Debug for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SendError::Disconnected(_) => f.debug_struct("SendError::Disconnected").finish(),
            SendError::Full(_) => f.debug_struct("SendError::Full").finish(),
        }
    }
}

impl<T> Error for SendError<T> {}

#[derive(Clone, PartialEq)]
pub enum BlockingSendError<T> {
    Disconnected(T),
}

impl<T> fmt::Debug for BlockingSendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockingSendError::Disconnected(_) => {
                f.debug_struct("BlockingSendError::Disconnected").finish()
            }
        }
    }
}

impl<T> fmt::Display for BlockingSendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockingSendError::Disconnected(_) => write!(f, "Channel Disconnected"),
        }
    }
}

impl<T> Error for BlockingSendError<T> {}

pub enum InnerRecvError {
    Disconnected,
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TryRecvError {
    Empty,
    Disconnected,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecvError {
    Disconnected,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SendError<T> {
    Disconnected(T),
    Full(T),
}

pub enum InnerRecvError {
    Disconnected,
    Empty,
    Invalid,
}

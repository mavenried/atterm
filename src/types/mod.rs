
mod app;
pub use app::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Error(String),
}

pub enum ReadState {
    Idle,
    WaitingReply,
}

use std::time::Duration;

mod app;
pub use app::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Error(String),
}

pub enum PendingItem {
    Command(String),
    Delay(Duration),
}

pub enum ReadState {
    Idle,
    WaitingReply,
    Delaying,
}

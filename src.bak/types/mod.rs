#[cfg(target_os = "windows")]
pub const DEFAULT_PORT: &str = "COM3";

#[cfg(not(target_os = "windows"))]
pub const DEFAULT_PORT: &str = "/dev/ttyUSB0";

pub const DEFAULT_BAUD: u32 = 115_200;
pub const READ_TIMEOUT_MS: u64 = 50;
pub const MAX_OUTPUT_LINES: usize = 500;
pub const COMMAND_PREFIX: char = ':';
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

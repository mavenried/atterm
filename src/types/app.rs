use std::collections::VecDeque;

use crate::constants::{COMMAND_PREFIX, MAX_OUTPUT_LINES};
use crate::types::{ConnectionStatus, PendingItem, ReadState};

pub struct App {
    pub output: Vec<String>,
    pub input: String,
    pub history: Vec<String>,
    pub history_cursor: Option<usize>,
    pub saved_input: String,
    pub cursor_pos: usize,
    pub status: ConnectionStatus,
    pub port_path: String,
    pub baud_rate: u32,
    pub scroll_offset: usize,
    pub auto_scroll: bool,
    pub pending: VecDeque<PendingItem>,
    pub read_state: ReadState,
    pub read_deadline: std::time::Instant,
}

impl App {
    pub fn new(port_path: &str, baud_rate: u32) -> Self {
        Self {
            output: Vec::new(),
            input: String::new(),
            history: Vec::new(),
            history_cursor: None,
            saved_input: String::new(),
            cursor_pos: 0,
            status: ConnectionStatus::Disconnected,
            port_path: port_path.to_owned(),
            baud_rate,
            scroll_offset: 0,
            auto_scroll: true,
            pending: VecDeque::new(),
            read_state: ReadState::Idle,
            read_deadline: std::time::Instant::now(),
        }
    }

    pub fn push_output(&mut self, line: impl Into<String>) {
        self.output.push(line.into());
        if self.output.len() > MAX_OUTPUT_LINES {
            self.output.drain(0..self.output.len() - MAX_OUTPUT_LINES);
        }
        if self.auto_scroll {
            self.scroll_offset = 0;
        }
    }

    pub fn commit_input(&mut self) -> Option<String> {
        let cmd = self.input.trim().to_owned();
        if cmd.is_empty() {
            return None;
        }
        self.push_output(format!("▶ {}", cmd));
        if self.history.last().map(|s| s.as_str()) != Some(&cmd) {
            self.history.push(cmd.clone());
        }
        self.history_cursor = None;
        self.saved_input.clear();
        self.input.clear();
        self.cursor_pos = 0;
        if cmd.starts_with(COMMAND_PREFIX) {
            None
        } else {
            Some(cmd)
        }
    }

    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        match self.history_cursor {
            None => {
                self.saved_input = self.input.clone();
                let idx = self.history.len() - 1;
                self.history_cursor = Some(idx);
                self.input = self.history[idx].clone();
            }
            Some(0) => {}
            Some(i) => {
                self.history_cursor = Some(i - 1);
                self.input = self.history[i - 1].clone();
            }
        }
        self.cursor_pos = self.input.len();
    }

    pub fn history_down(&mut self) {
        match self.history_cursor {
            None => {}
            Some(i) if i + 1 >= self.history.len() => {
                self.history_cursor = None;
                self.input = self.saved_input.clone();
                self.cursor_pos = self.input.len();
            }
            Some(i) => {
                self.history_cursor = Some(i + 1);
                self.input = self.history[i + 1].clone();
                self.cursor_pos = self.input.len();
            }
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor_pos, c);
        self.cursor_pos += c.len_utf8();
    }

    pub fn delete_char_before(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }
        let prev = self.input[..self.cursor_pos]
            .char_indices()
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.input.remove(prev);
        self.cursor_pos = prev;
    }

    pub fn delete_char_after(&mut self) {
        if self.cursor_pos < self.input.len() {
            self.input.remove(self.cursor_pos);
        }
    }
}

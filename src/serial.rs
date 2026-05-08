//! Serial port management: opening, the reader thread, and the rx drain + read-state machine.

use std::{
    io::{self, Read, Write},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use serialport::SerialPort;

use crate::{
    constants::{READ_TIMEOUT_MS, REPLY_TIMEOUT_MS},
    protocol::{frame_command, is_terminal_response},
    types::{App, ConnectionStatus, PendingItem, ReadState},
};

/// Open the serial port, spawn the reader thread, return the write handle.
pub fn open_port(
    port_path: &str,
    baud_rate: u32,
    rx_buf: Arc<Mutex<Vec<String>>>,
    status: Arc<Mutex<ConnectionStatus>>,
) -> Result<Option<Arc<Mutex<Box<dyn SerialPort>>>>> {
    match serialport::new(port_path, baud_rate)
        .timeout(Duration::from_millis(READ_TIMEOUT_MS))
        .open()
    {
        Ok(port) => {
            let reader = port.try_clone().context("failed to clone serial port")?;
            *status.lock().unwrap() = ConnectionStatus::Connected;
            spawn_reader(reader, rx_buf, status);
            Ok(Some(Arc::new(Mutex::new(port))))
        }
        Err(e) => {
            *status.lock().unwrap() = ConnectionStatus::Error(format!("open failed: {e}"));
            Ok(None)
        }
    }
}

/// Spawn the background thread that reads bytes from the port and assembles lines.
pub fn spawn_reader(
    port: Box<dyn SerialPort>,
    rx_buf: Arc<Mutex<Vec<String>>>,
    status: Arc<Mutex<ConnectionStatus>>,
) {
    thread::spawn(move || {
        let mut port = port;
        let mut line_buf = String::new();
        let mut byte_buf = [0u8; 256];

        loop {
            match port.read(&mut byte_buf) {
                Ok(0) => thread::sleep(Duration::from_millis(READ_TIMEOUT_MS)),
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&byte_buf[..n]);
                    for ch in chunk.chars() {
                        match ch {
                            '\r' => {}
                            '\n' => {
                                let line = std::mem::take(&mut line_buf);
                                if !line.is_empty() {
                                    rx_buf.lock().unwrap().push(line);
                                }
                            }
                            c => line_buf.push(c),
                        }
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::TimedOut => {}
                Err(e) => {
                    *status.lock().unwrap() = ConnectionStatus::Error(format!("read error: {e}"));
                    break;
                }
            }
        }
    });
}

/// Drain the rx buffer into app output, advancing the read-state machine on terminal responses.
pub fn drain_rx(app: &mut App, rx_buf: &Arc<Mutex<Vec<String>>>) {
    let prev_len = app.output.len();
    {
        let mut buf = rx_buf.lock().unwrap();
        for line in buf.drain(..) {
            app.push_output(line);
        }
    }
    if matches!(app.read_state, ReadState::WaitingReply) {
        let new_lines = &app.output[prev_len..];
        if new_lines.iter().any(|l| is_terminal_response(l)) {
            app.read_state = ReadState::Idle;
        } else if app.output.len() > prev_len {
            app.read_deadline = Instant::now() + Duration::from_millis(REPLY_TIMEOUT_MS);
        }
    }
}

/// Advance the pending-command queue, sending the next command when the device is ready.
pub fn tick_pending(app: &mut App, write_port: &Option<Arc<Mutex<Box<dyn SerialPort>>>>) {
    match app.read_state {
        ReadState::Idle => {
            if let Some(item) = app.pending.pop_front() {
                match item {
                    PendingItem::Delay(dur) => {
                        app.push_output(format!("⧗ delay {}ms", dur.as_millis()));
                        app.read_state = ReadState::Delaying;
                        app.read_deadline = Instant::now() + dur;
                    }
                    PendingItem::Command(cmd) => {
                        if let Some(ref port_lock) = write_port {
                            app.push_output(format!("▶ {}", cmd));
                            let mut port = port_lock.lock().unwrap();
                            let payload = frame_command(&cmd);
                            if let Err(e) = port.write_all(payload.as_bytes()) {
                                app.push_output(format!("✖ write error: {e}"));
                                app.pending.clear();
                            } else {
                                let _ = port.flush();
                                app.history.push(cmd);
                                app.read_state = ReadState::WaitingReply;
                                app.read_deadline =
                                    Instant::now() + Duration::from_millis(REPLY_TIMEOUT_MS);
                            }
                        }
                    }
                }
            }
        }
        ReadState::Delaying => {
            if Instant::now() >= app.read_deadline {
                app.read_state = ReadState::Idle;
            }
        }
        ReadState::WaitingReply => {
            if Instant::now() >= app.read_deadline {
                app.push_output("✖ :read timeout waiting for reply, aborting".to_owned());
                app.read_state = ReadState::Idle;
            }
        }
    }
}

/// Write a single AT command immediately (interactive input, not queued).
pub fn send_command(
    app: &mut App,
    cmd: &str,
    write_port: &Option<Arc<Mutex<Box<dyn SerialPort>>>>,
) {
    if let Some(ref port_lock) = write_port {
        let mut port = port_lock.lock().unwrap();
        let payload = frame_command(cmd);
        if let Err(e) = port.write_all(payload.as_bytes()) {
            app.push_output(format!("✖ write error: {e}"));
        } else {
            let _ = port.flush();
        }
    } else {
        app.push_output("✖ not connected".to_owned());
    }
}

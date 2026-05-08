use std::{
    io,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use serialport::SerialPort;

use crate::types::*;

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

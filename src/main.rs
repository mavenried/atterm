mod commands;
mod constants;
mod input;
mod protocol;
mod serial;
mod types;
mod ui;

use std::{
    io,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use constants::{DEFAULT_BAUD, DEFAULT_PORT};
use types::{App, ConnectionStatus};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let port_path = args.get(1).map(|s| s.as_str()).unwrap_or(DEFAULT_PORT);
    let baud_rate: u32 = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_BAUD);

    let rx_buf = Arc::new(Mutex::new(Vec::<String>::new()));
    let serial_status = Arc::new(Mutex::new(ConnectionStatus::Disconnected));

    let write_port = serial::open_port(
        port_path,
        baud_rate,
        Arc::clone(&rx_buf),
        Arc::clone(&serial_status),
    )?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(port_path, baud_rate);
    app.status = serial_status.lock().unwrap().clone();

    loop {
        serial::drain_rx(&mut app, &rx_buf);
        app.status = serial_status.lock().unwrap().clone();
        serial::tick_pending(&mut app, &write_port);

        terminal.draw(|f| ui::ui(f, &app))?;

        if input::handle(&mut app, &write_port)? {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

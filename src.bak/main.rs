mod helpers;
mod input;
mod prelude;
mod reader;
mod types;
mod ui;

use prelude::*;
use reader::*;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let port_path = args.get(1).map(|s| s.as_str()).unwrap_or(DEFAULT_PORT);
    let baud_rate: u32 = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_BAUD);

    let rx_buf: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let serial_status: Arc<Mutex<ConnectionStatus>> =
        Arc::new(Mutex::new(ConnectionStatus::Disconnected));

    let write_port: Option<Arc<Mutex<Box<dyn SerialPort>>>> =
        match serialport::new(port_path, baud_rate)
            .timeout(Duration::from_millis(READ_TIMEOUT_MS))
            .open()
        {
            Ok(port) => {
                let reader = port.try_clone().context("failed to clone serial port")?;
                *serial_status.lock().unwrap() = ConnectionStatus::Connected;
                spawn_reader(reader, Arc::clone(&rx_buf), Arc::clone(&serial_status));
                Some(Arc::new(Mutex::new(port)))
            }
            Err(e) => {
                *serial_status.lock().unwrap() =
                    ConnectionStatus::Error(format!("open failed: {e}"));
                None
            }
        };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(port_path, baud_rate);
    app.status = serial_status.lock().unwrap().clone();

    loop {
        {
            let prev_len = app.output.len();
            let mut buf = rx_buf.lock().unwrap();
            for line in buf.drain(..) {
                app.push_output(line);
            }
            if matches!(app.read_state, ReadState::WaitingReply) {
                let new_lines = &app.output[prev_len..];
                if new_lines.iter().any(|l| helpers::is_terminal_response(l)) {
                    app.read_state = ReadState::Idle;
                } else if app.output.len() > prev_len {
                    app.read_deadline = std::time::Instant::now() + Duration::from_millis(5000);
                }
            }
        }

        {
            app.status = serial_status.lock().unwrap().clone();
        }

        match app.read_state {
            ReadState::Idle => {
                if let Some(cmd) = app.pending.pop_front() {
                    if let Some(ref port_lock) = write_port {
                        app.push_output(format!("▶ {}", cmd));
                        let mut port = port_lock.lock().unwrap();
                        let payload = format!("{}\r\n", cmd);
                        if let Err(e) = port.write_all(payload.as_bytes()) {
                            app.push_output(format!("✖ write error: {e}"));
                            app.pending.clear();
                        } else {
                            let _ = port.flush();
                            app.history.push(cmd.clone());
                            app.read_state = ReadState::WaitingReply;
                            app.read_deadline =
                                std::time::Instant::now() + Duration::from_millis(5000);
                        }
                    }
                }
            }
            ReadState::WaitingReply => {
                if std::time::Instant::now() >= app.read_deadline {
                    app.push_output("✖ :read timeout waiting for reply, aborting".to_owned());
                    // app.pending.clear();
                    app.read_state = ReadState::Idle;
                }
            }
        }

        terminal.draw(|f| ui::ui(f, &app))?;

        if input::input_handing(&mut app, &write_port)? {
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

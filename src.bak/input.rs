use crate::prelude::*;
use crate::types::*;

pub fn input_handing(
    app: &mut App,
    write_port: &Option<Arc<Mutex<Box<dyn SerialPort>>>>,
) -> Result<bool> {
    if !event::poll(Duration::from_millis(READ_TIMEOUT_MS))? {
        return Ok(false);
    }

    match event::read()? {
        Event::Key(key) => {
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                return Ok(true);
            }

            match key.code {
                KeyCode::Enter => {
                    let raw = app.input.trim().to_owned();
                    if let Some(cmd) = app.commit_input() {
                        // Normal AT command : send to port
                        if let Some(ref port_lock) = write_port {
                            let mut port = port_lock.lock().unwrap();
                            let payload = format!("{}\r\n", cmd);
                            if let Err(e) = port.write_all(payload.as_bytes()) {
                                app.push_output(format!("✖ write error: {e}"));
                                Ok(false)
                            } else {
                                let _ = port.flush();
                                Ok(false)
                            }
                        } else {
                            app.push_output("✖ not connected".to_owned());
                            Ok(false)
                        }
                    } else if raw.starts_with(COMMAND_PREFIX) {
                        // Internal command
                        match App::parse_internal(&raw) {
                            ("exit", _) => return Ok(true),

                            ("read", Some(path)) => {
                                match std::fs::read_to_string(path) {
                                    Err(e) => {
                                        app.push_output(format!("✖ read: {e}"));
                                    }
                                    Ok(contents) => {
                                        let cmds: Vec<String> = contents
                                            .lines()
                                            .map(|l| l.trim().to_owned())
                                            .filter(|l| !l.is_empty() && !l.starts_with('#'))
                                            .collect();
                                        app.push_output(format!(
                                            "✔ queuing {} commands from {}",
                                            cmds.len(),
                                            path
                                        ));
                                        for c in cmds {
                                            app.pending.push_back(c);
                                        }
                                    }
                                }
                                Ok(false)
                            }
                            ("read", None) => {
                                app.push_output("✖ usage: :read <file>");
                                Ok(false)
                            }

                            ("save", Some(path)) => {
                                // save only non-internal commands
                                let cmds: Vec<&String> = app
                                    .history
                                    .iter()
                                    .filter(|c| !c.starts_with(COMMAND_PREFIX))
                                    .collect();
                                let contents = cmds
                                    .iter()
                                    .map(|s| s.as_str())
                                    .collect::<Vec<_>>()
                                    .join("\n")
                                    + "\n";
                                match std::fs::write(path, contents) {
                                    Ok(_) => app.push_output(format!(
                                        "✔ saved {} commands to {}",
                                        cmds.len(),
                                        path
                                    )),
                                    Err(e) => app.push_output(format!("✖ save: {e}")),
                                }
                                Ok(false)
                            }
                            ("save", None) => {
                                app.push_output("✖ usage: :save <file>");
                                return Ok(false);
                            }

                            ("clear", _) => {
                                app.output.clear();
                                app.scroll_offset = 0;
                                Ok(false)
                            }

                            (unknown, _) => {
                                app.push_output(format!("✖ unknown command: :{}", unknown));
                                Ok(false)
                            }
                        }
                    } else {
                        Ok(false)
                    }
                }
                KeyCode::Up => {
                    app.history_up();
                    Ok(false)
                }
                KeyCode::Down => {
                    app.history_down();
                    Ok(false)
                }
                KeyCode::Left => {
                    if app.cursor_pos > 0 {
                        app.cursor_pos -= 1;
                    }
                    Ok(false)
                }
                KeyCode::Right => {
                    if app.cursor_pos < app.input.len() {
                        app.cursor_pos += 1;
                    }
                    Ok(false)
                }
                KeyCode::Home => {
                    app.cursor_pos = 0;
                    Ok(false)
                }
                KeyCode::End => {
                    app.cursor_pos = app.input.len();
                    Ok(false)
                }
                KeyCode::Backspace => {
                    app.delete_char_before();
                    Ok(false)
                }
                KeyCode::Delete => {
                    app.delete_char_after();
                    Ok(false)
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    app.input.clear();
                    app.cursor_pos = 0;
                    Ok(false)
                }
                KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    while app.cursor_pos > 0 {
                        app.delete_char_before();
                        if app.cursor_pos > 0 && app.input[..app.cursor_pos].ends_with(' ') {
                            break;
                        }
                    }
                    Ok(false)
                }
                KeyCode::Char(c) => {
                    app.insert_char(c);
                    Ok(false)
                }
                KeyCode::PageUp => {
                    app.auto_scroll = false;
                    app.scroll_offset =
                        (app.scroll_offset + 10).min(app.output.len().saturating_sub(1));
                    Ok(false)
                }
                KeyCode::PageDown => {
                    if app.scroll_offset > 10 {
                        app.scroll_offset -= 10;
                    } else {
                        app.scroll_offset = 0;
                        app.auto_scroll = true;
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        Event::Mouse(mouse) => {
            match mouse.kind {
                MouseEventKind::ScrollUp => {
                    app.auto_scroll = false;
                    app.scroll_offset =
                        (app.scroll_offset + 3).min(app.output.len().saturating_sub(1));
                }
                MouseEventKind::ScrollDown => {
                    if app.scroll_offset > 3 {
                        app.scroll_offset -= 3;
                    } else {
                        app.scroll_offset = 0;
                        app.auto_scroll = true;
                    }
                }
                _ => {}
            };
            Ok(false)
        }
        _ => Ok(false),
    }
}

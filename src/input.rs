//! Keyboard and mouse event handling.

use std::{sync::{Arc, Mutex}, time::Duration};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEventKind};
use serialport::SerialPort;

use crate::{
    commands,
    constants::{COMMAND_PREFIX, READ_TIMEOUT_MS},
    serial::send_command,
    types::App,
};

/// Poll for and handle one event. Returns true if the app should exit.
pub fn handle(
    app: &mut App,
    write_port: &Option<Arc<Mutex<Box<dyn SerialPort>>>>,
) -> Result<bool> {
    if !event::poll(Duration::from_millis(READ_TIMEOUT_MS))? {
        return Ok(false);
    }

    match event::read()? {
        Event::Key(key) => handle_key(app, write_port, key),
        Event::Mouse(mouse) => {
            handle_mouse(app, mouse);
            Ok(false)
        }
        _ => Ok(false),
    }
}

fn handle_key(
    app: &mut App,
    write_port: &Option<Arc<Mutex<Box<dyn SerialPort>>>>,
    key: crossterm::event::KeyEvent,
) -> Result<bool> {
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Ok(true);
    }

    match key.code {
        KeyCode::Enter => {
            let raw = app.input.trim().to_owned();
            if let Some(cmd) = app.commit_input() {
                send_command(app, &cmd, write_port);
            } else if raw.starts_with(COMMAND_PREFIX) {
                if commands::dispatch(app, &raw, write_port) {
                    return Ok(true);
                }
            }
        }

        KeyCode::Up => app.history_up(),
        KeyCode::Down => app.history_down(),

        KeyCode::Left => {
            if app.cursor_pos > 0 {
                app.cursor_pos -= 1;
            }
        }
        KeyCode::Right => {
            if app.cursor_pos < app.input.len() {
                app.cursor_pos += 1;
            }
        }
        KeyCode::Home => app.cursor_pos = 0,
        KeyCode::End => app.cursor_pos = app.input.len(),

        KeyCode::Backspace => app.delete_char_before(),
        KeyCode::Delete => app.delete_char_after(),

        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.input.clear();
            app.cursor_pos = 0;
        }
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            while app.cursor_pos > 0 {
                app.delete_char_before();
                if app.cursor_pos > 0 && app.input[..app.cursor_pos].ends_with(' ') {
                    break;
                }
            }
        }

        KeyCode::Char(c) => app.insert_char(c),

        KeyCode::PageUp => {
            app.auto_scroll = false;
            app.scroll_offset =
                (app.scroll_offset + 10).min(app.output.len().saturating_sub(1));
        }
        KeyCode::PageDown => {
            if app.scroll_offset > 10 {
                app.scroll_offset -= 10;
            } else {
                app.scroll_offset = 0;
                app.auto_scroll = true;
            }
        }

        _ => {}
    }

    Ok(false)
}

fn handle_mouse(app: &mut App, mouse: crossterm::event::MouseEvent) {
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
    }
}

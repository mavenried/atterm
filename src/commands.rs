//! Internal terminal commands (`:exit`, `:read`, `:save`, `:clear`).

use std::{sync::{Arc, Mutex}, time::Duration};

use serialport::SerialPort;

use crate::{
    constants::COMMAND_PREFIX,
    types::{App, PendingItem},
};

/// Dispatch a parsed internal command. Returns true if the app should exit.
pub fn dispatch(
    app: &mut App,
    raw: &str,
    write_port: &Option<Arc<Mutex<Box<dyn SerialPort>>>>,
) -> bool {
    match parse(raw) {
        ("exit", _) => return true,

        ("read", Some(path)) => cmd_read(app, path),
        ("read", None) => app.push_output("✖ usage: :read <file>"),

        ("save", Some(path)) => cmd_save(app, path),
        ("save", None) => app.push_output("✖ usage: :save <file>"),

        ("clear", _) => {
            app.output.clear();
            app.scroll_offset = 0;
        }

        (unknown, _) => app.push_output(format!("✖ unknown command: :{}", unknown)),
    }

    let _ = write_port;
    false
}

/// Split `:verb [arg]` — returns (verb, Option<arg>).
fn parse(cmd: &str) -> (&str, Option<&str>) {
    let body = cmd.trim_start_matches(COMMAND_PREFIX).trim();
    match body.split_once(char::is_whitespace) {
        Some((verb, arg)) => (verb, Some(arg.trim())),
        None => (body, None),
    }
}

fn cmd_read(app: &mut App, path: &str) {
    let path = if std::path::Path::new(path).extension().is_none() {
        format!("{}.at", path)
    } else {
        path.to_owned()
    };

    match std::fs::read_to_string(&path) {
        Err(e) => app.push_output(format!("✖ read: {e}")),
        Ok(contents) => {
            let items: Vec<PendingItem> = contents
                .lines()
                .map(|l| l.trim().to_owned())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .map(|l| {
                    if let Some(ms) = l
                        .strip_prefix(":delay ")
                        .and_then(|s| s.trim().parse::<u64>().ok())
                    {
                        PendingItem::Delay(Duration::from_millis(ms))
                    } else {
                        PendingItem::Command(l)
                    }
                })
                .collect();
            app.push_output(format!("✔ queuing {} items from {}", items.len(), path));
            for item in items {
                app.pending.push_back(item);
            }
        }
    }
}

fn cmd_save(app: &mut App, path: &str) {
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
        Ok(_) => app.push_output(format!("✔ saved {} commands to {}", cmds.len(), path)),
        Err(e) => app.push_output(format!("✖ save: {e}")),
    }
}

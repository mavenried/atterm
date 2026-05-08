# atterm

A terminal UI for interacting with AT command IoT devices over serial. Built with [ratatui](https://github.com/ratatui-org/ratatui) and [serialport](https://github.com/serialport/serialport-rs).

```
╭─ AT Terminal ────────────────────────────────────────────────────────────────╮
│ /dev/ttyUSB0 @ 115200baud  ● CONNECTED  AUTO-SCROLL  [^C] quit  [↑↓] history │
╰──────────────────────────────────────────────────────────────────────────────╯
╭─ Output (12 lines) ──────────────────────────────────────────────────────────╮
│ ▶ AT                                                                        │
│ OK                                                                           │
│ ▶ AT+GMR                                                                    │
│ AT version:2.4.0.0                                                           │
│ SDK version:v4.3.2                                                           │
│ OK                                                                           │
╰──────────────────────────────────────────────────────────────────────────────╯
╭─ Command (Enter → send \n\r) ────────────────────────────────────────────────╮
│ AT+CWMODE=1                                                                  │
╰──────────────────────────────────────────────────────────────────────────────╯
```

## Features

- Send AT commands with correct `\r\n` framing
- Scrollable output with mouse wheel and PgUp/PgDn
- Command history browsed with arrow keys
- Color-coded output — sent commands in yellow, `OK` and terminal responses in green, errors in red
- Input border turns magenta while waiting for a reply
- Queued command counter shown in the header during script playback
- Graceful degradation — launches and shows the error if the port fails to open

## Build

Requires `libudev-dev` on Debian/Ubuntu:

```bash
sudo apt install libudev-dev pkg-config
```

```bash
cargo build --release
```

## Usage

```bash
# Default: /dev/ttyUSB0 at 115200 baud
./at-terminal

# Custom port and baud rate
./at-terminal /dev/ttyUSB1 9600
```

If your user can't open the serial port:

```bash
sudo usermod -aG dialout $USER
# then log out and back in
```

## Keybindings

| Key | Action |
|-----|--------|
| `Enter` | Send command |
| `↑` / `↓` | Browse command history |
| `←` / `→` | Move cursor |
| `Home` / `End` | Jump to start/end of input |
| `Backspace` / `Del` | Delete character |
| `^U` | Clear input line |
| `^W` | Delete word before cursor |
| `PgUp` / `PgDn` | Scroll output (10 lines) |
| Scroll wheel | Scroll output (3 lines) |
| `^C` | Quit |

Scrolling up disables auto-scroll. It re-enables when you reach the bottom.

## Internal Commands

Internal commands start with `:` and control the terminal itself rather than the device.

### `:exit`
Quit the terminal.

### `:read <file>`
Load a `.at` script and execute each command in sequence, waiting for a terminal response (`OK`, `ERROR`, `NO CARRIER`, etc.) before sending the next one. If no extension is given, `.at` is appended automatically.

```bash
:read init
:read /home/user/scripts/connect.at
```

A 5 second inactivity timeout aborts the queue if the device stops responding. The timer resets on each received line, so commands that stream a lot of output won't be cut off prematurely.

### `:save <file>`
Save all AT commands from the current session history to a file. Internal commands (`:read`, `:save`, etc.) are excluded, so the output is a valid script that can be replayed with `:read`.

```bash
:save session.at
```

### `:clear`
Clear the output pane.

## Script Format (`.at` files)

Plain text, one command per line. Lines starting with `#` are treated as comments and skipped. Blank lines are ignored.

```
# Initialize device
AT

# Configure WiFi
AT+CWMODE=1
AT+CWJAP="ssid","password"

# Check IP
AT+CIFSR
```

Use `:delay <ms>` inside a script to insert a pause — useful after commands like `AT+RST` that reboot the device and continue sending output after `OK`:

```
AT+RST
:delay 3000
AT+CWMODE=1
```

## Output Colors

| Color | Meaning |
|-------|---------|
| Yellow | Command sent (`▶ AT+GMR`) |
| Green | Terminal response (`OK`, `NO CARRIER`, `+CME ERROR`, etc.) |
| Red | Error (`✖ write error`, `ERROR` responses) |
| White | Normal device output |

## Project Structure

```
src/
├── main.rs          # Startup, event loop, terminal setup/teardown
├── constants.rs     # Port defaults, timeouts, buffer sizes
├── protocol.rs      # AT framing, terminal response detection
├── serial.rs        # Port open, reader thread, rx drain, pending queue
├── commands.rs      # Internal :command dispatch
├── input.rs         # Keyboard and mouse event handling
├── ui.rs            # Ratatui rendering
└── types/
    ├── mod.rs       # ConnectionStatus, ReadState enums
    └── app.rs       # App state and input editing methods
```

## Dependencies

```toml
ratatui = "0.26"
crossterm = "0.27"
serialport = "4.6"
anyhow = "1.0"
```

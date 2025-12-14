# SerialTUI

A high-performance terminal UI for serial port communication with scripting support, built in Rust.

## Features

- **Multi-Port Support** - Connect to up to 255 serial ports simultaneously
- **Interleaved Display** - View all port data in one stream with per-port coloring
- **Vim-Style Navigation** - `j/k`, `gg/G`, `/search`, `y` yank to clipboard
- **Custom Scripting** - Automate with `.stui` scripts (Rust-like syntax)
- **Auto-Reconnection** - Exponential backoff reconnect on disconnect
- **Comprehensive Logging** - Combined log + per-port log files
- **Regex Search** - ripgrep-powered search with highlighting

## Quick Start

```bash
cargo build --release
./target/release/serial_tui
```

## Screenshots

```
┌─────────────────────────────────────────────────────────────────────┐
│ [Port v] [Filter v] [Add Port] [Run Script v]          Connected   │
├─────────────────────────────────────────────────────────────────────┤
│ <14:32:01.123>[GPS] NMEA: $GPGGA,123456.00,4807.038,N...            │
│ <14:32:01.456>[Motor] RPM: 1500, Temp: 45C                          │
│ <14:32:01.789>[GPS] NMEA: $GPRMC,123456.00,A,4807.038...            │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│ [x] GPS   │ > PING█                                                 │
│ [ ] Motor │                                                         │
└───────────┴─────────────────────────────────────────────────────────┘
```

## Documentation

| Document | Description |
|----------|-------------|
| [Design Guide](DESIGN.md) | Step-by-step implementation guide with code examples |
| [Architecture](ARCHITECTURE.md) | Detailed system architecture and data flow |
| [Todo / Kanban](todo.md) | Task tracking and milestones |
| [Learning Guide](learn.md) | Prerequisites and resources for contributors |

## Usage

### Keybindings

| Key | Action |
|-----|--------|
| `j` / `k` | Scroll down / up |
| `gg` / `G` | Jump to top / bottom |
| `Ctrl+d` / `Ctrl+u` | Half-page down / up |
| `/` | Search (regex) |
| `n` / `N` | Next / previous match |
| `y` / `yy` | Yank selection / line |
| `i` | Insert mode (input box) |
| `Esc` | Normal mode |

### Commands

| Command | Description |
|---------|-------------|
| `:q` | Quit |
| `:w` | Save config |
| `:run <script>` | Run a .stui script |
| `:stop` | Abort running script |
| `:debug` | Toggle debug screen |
| `:clear` | Clear display buffer |

### Configuration

Create `config/config.toml`:

```toml
[[port]]
name = "GPS"
path = "/dev/ttyUSB0"
baud_rate = 115200
color = "#FF5733"

[[port]]
name = "Motor"
path = "/dev/ttyUSB1"
baud_rate = 9600
line_ending = "\r\n"
```

### Scripting

Create scripts in `scripts/` with `.stui` extension:

```rust
// scripts/test.stui
let ports = ["GPS", "Motor"];

for port in ports {
    sendstr([port], "INIT\n");
    wait(0.5);
}

// Wait for response with timeout
waitstr(["GPS"], r"OK|ACK", 5.0);

// Send binary data
sendbin(["Motor"], "0x01020304");
```

Run with `:run test.stui` or from the UI dropdown.

## Project Status

**Status: In Development**

See [todo.md](todo.md) for current progress and [DESIGN.md](DESIGN.md) for implementation roadmap.

### Milestones

- [ ] M1: Foundation + Basic Serial
- [ ] M2: Minimal TUI + Channel Wiring
- [ ] M3: Multi-Port + Notifications
- [ ] M4: Vim Navigation + Preview
- [ ] M5: Config UI + Commands
- [ ] M6: Script Engine
- [ ] M7: Polish + Advanced Features

## Tech Stack

- **Async Runtime**: [tokio](https://tokio.rs/)
- **TUI Framework**: [ratatui](https://ratatui.rs/)
- **Serial**: [tokio-serial](https://docs.rs/tokio-serial/)
- **Config**: [serde](https://serde.rs/) + [toml](https://docs.rs/toml/)

## License

MIT

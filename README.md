# SerialTUI

Terminal UI for multi-port serial communication with vim navigation and scripting.

## Features

- **Multi-Port** - Up to 255 ports simultaneously
- **Vim Navigation** - `j/k`, `gg/G`, `/search`, `y` yank
- **Scripting** - Automate with `.stui` scripts
- **Auto-Reconnect** - Exponential backoff on disconnect
- **Logging** - Combined + per-port log files

## Quick Start

```bash
cargo build --release
./target/release/serial_tui
```

## Keybindings

| Key | Action |
|-----|--------|
| `j`/`k` | Scroll down/up |
| `gg`/`G` | Top/bottom |
| `Ctrl+d`/`u` | Half-page |
| `/` | Search |
| `n`/`N` | Next/prev match |
| `y`/`yy` | Yank |
| `i` | Insert mode |
| `:q` | Quit |
| `:w` | Save config |
| `:run <script>` | Run script |

## Configuration

`config/config.toml`:
```toml
[[port]]
name = "GPS"
path = "/dev/ttyUSB0"
baud_rate = 115200
color = "#FF5733"
```

## Scripting

`scripts/test.stui`:
```
sendstr(["GPS"], "PING\n")
wait(0.5)
waitstr(["GPS"], r"OK|ACK", 5.0)
```

## Documentation

- [Architecture](ARCHITECTURE.md) - System design and data flow
- [Todo](todo.md) - Task tracking
- [Learn](learn.md) - Prerequisites for contributors

## Status

In development. See [todo.md](todo.md) for progress.

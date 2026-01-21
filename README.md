# SerialTUI

Terminal UI for multi-port serial communication with vim-style navigation.

## Features

- **Multi-Port Monitoring** - Monitor multiple serial ports with per-port colors
- **Vim Navigation** - `j/k`, `gg/G`, `Ctrl+u/d`, `/search`, `n/N`
- **Visual Selection** - `v` to select, `y` to yank to clipboard
- **Line Buffering** - Complete lines only (splits on `\n` or `\r`)
- **Configurable** - TOML config with colors and line endings

## Quick Start

```bash
cargo build --release
./target/release/serial_tui
```

On first run, creates `config/ports.toml`:

```toml
[device1]
path = "/dev/ttyUSB0"    # Linux
# path = "COM3"          # Windows
baud_rate = 115200
line_ending = "lf"       # lf, cr, or crlf
color = "green"          # Named or "#RRGGBB"
```

## Keybindings

| Key | Action |
|-----|--------|
| `Tab` | Cycle focus |
| `Esc` | Quit |
| `j/k` | Scroll down/up |
| `gg/G` | Top/bottom |
| `Ctrl+d/u` | Half-page |
| `/` | Search |
| `n/N` | Next/prev match |
| `v` | Visual select |
| `y` | Yank to clipboard |
| `Ctrl+Space` | Select send ports |
| `Enter` | Send text |

## Layout

```
+----------------------------------------------------------+
| Config: [p] Ports                                         |
+----------------------------------------------------------+
| [12:34:56.789] [com1] Hello from port 1                  |
| [12:34:56.790] [com2] Response from port 2               |
+----------------------------------------------------------+
| Input: [ports] type here_                                 |
+----------------------------------------------------------+
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for technical details.

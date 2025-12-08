# SerialTUI

A high-performance terminal UI for serial port communication with scripting support.

## Features

- Connect to up to 255 serial ports simultaneously
- Interleaved display with per-port coloring
- Vim-style navigation and editing
- Custom scripting language (.stui) with Rust-like syntax
- Auto-reconnection on disconnect
- Comprehensive logging (combined + per-port)
- Regex-powered search (ripgrep engine)

---

## Directory Structure

```
serialtui/
├── Cargo.toml
├── config/
│   ├── config.toml          # Main port config
│   └── macros.toml           # Vim keybind macros
├── scripts/
│   └── example.stui          # User scripts
├── logs/
│   ├── all.log               # Combined log
│   ├── COM1.log              # Per-port logs
│   └── ...
└── src/
    ├── main.rs
    ├── app.rs                # App state, main event loop
    │
    ├── serial/
    │   ├── mod.rs
    │   ├── handler.rs        # Async serial read/write tasks
    │   ├── port.rs           # Port struct, config, state
    │   └── message.rs        # SerialMessage { timestamp, port_name, data }
    │
    ├── config/
    │   ├── mod.rs
    │   ├── manager.rs        # Load/save/validate config
    │   ├── port_config.rs    # Port configuration struct
    │   └── macro_config.rs   # Vim macro definitions
    │
    ├── ui/
    │   ├── mod.rs
    │   ├── app_ui.rs         # Main layout (top/middle/bottom)
    │   ├── config_bar.rs     # Top: port dropdown, filters, add port, run script
    │   ├── display.rs        # Middle: interleaved serial output
    │   ├── preview.rs        # Bottom of display: wrapped line preview
    │   ├── input_box.rs      # Bottom: vim-style text input
    │   ├── port_selector.rs  # Bottom-left: checkboxes for send targets
    │   ├── popup/
    │   │   ├── mod.rs
    │   │   ├── add_port.rs   # Add port popup form
    │   │   ├── color_picker.rs
    │   │   └── confirm.rs    # Exit confirmation
    │   ├── notification.rs   # Top-right stacking notifications
    │   ├── debug_screen.rs   # :debug screen overlay
    │   └── vim/
    │       ├── mod.rs
    │       ├── navigation.rs # j/k/gg/G/Ctrl-d/Ctrl-u
    │       ├── search.rs     # /, n, N, highlighting
    │       ├── yank.rs       # y, yy -> clipboard
    │       └── input_mode.rs # Normal/insert mode for input box
    │
    ├── script/
    │   ├── mod.rs
    │   ├── lexer.rs          # Tokenizer
    │   ├── parser.rs         # AST generation
    │   ├── ast.rs            # AST node definitions
    │   ├── interpreter.rs    # Execute AST
    │   ├── builtins.rs       # sendstr, sendbin, wait, waitstr
    │   └── error.rs          # Script errors
    │
    ├── logger/
    │   ├── mod.rs
    │   ├── writer.rs         # Async log file writer
    │   └── formatter.rs      # <timestamp>[<port_name>] format
    │
    └── notification/
        ├── mod.rs
        └── system.rs         # Notification queue, timing, display
```

---

## Architecture

### Data Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              MAIN EVENT LOOP                                │
│                         (tokio runtime + ratatui)                           │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
          ┌───────────────────────────┼───────────────────────────┐
          ▼                           ▼                           ▼
   ┌─────────────┐            ┌──────────────┐           ┌────────────────┐
   │   UI Task   │◄──────────►│  App State   │◄─────────►│ Script Engine  │
   │  (ratatui)  │            │  (Arc<...>)  │           │    (async)     │
   └─────────────┘            └──────────────┘           └────────────────┘
          │                           ▲                           │
          │ crossterm                 │                           │
          │ events                    │                           │
          ▼                           │                           ▼
   ┌─────────────┐                    │                  ┌────────────────┐
   │   Input     │────────────────────┘                  │   Builtins     │
   │  Handler    │                                       │ sendstr/wait/  │
   └─────────────┘                                       │   waitstr      │
                                                         └────────────────┘
                                                                  │
                                                                  ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                            SERIAL HANDLER                                   │
│                    (tokio tasks, one per port)                              │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐       ┌─────────┐                   │
│  │  COM1   │  │  COM2   │  │  COM3   │  ...  │ COM255  │                   │
│  │  task   │  │  task   │  │  task   │       │  task   │                   │
│  └────┬────┘  └────┬────┘  └────┬────┘       └────┬────┘                   │
│       │            │            │                  │                        │
│       └────────────┴────────────┴──────────────────┘                        │
│                              │                                              │
│                    tokio::sync::broadcast                                   │
│                      (fan-out to all)                                       │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                    ┌─────────────────┼─────────────────┐
                    ▼                 ▼                 ▼
            ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
            │   Display    │  │    Logger    │  │   Script     │
            │   Buffer     │  │   (async)    │  │   waitstr    │
            │              │  │              │  │   listener   │
            └──────────────┘  └──────────────┘  └──────────────┘
                                      │
                                      ▼
                              ┌──────────────┐
                              │  Log Files   │
                              │  all.log     │
                              │  <port>.log  │
                              └──────────────┘
```

### Channel Architecture

```rust
// Channels for communication between components

// Serial read -> broadcast to all subscribers
serial_broadcast: tokio::sync::broadcast::Sender<SerialMessage>

// UI/Script -> Serial write
serial_tx: tokio::sync::mpsc::Sender<SerialCommand>

pub enum SerialCommand {
    Send { port_names: Vec<String>, data: Vec<u8> },
    Connect { port_name: String },
    Disconnect { port_name: String },
}

// Notifications
notification_tx: tokio::sync::mpsc::Sender<Notification>

// Script control
script_tx: tokio::sync::mpsc::Sender<ScriptCommand>

pub enum ScriptCommand {
    Run { path: PathBuf },
    Abort,
}
```

---

## Key Types

```rust
// src/serial/message.rs
pub struct SerialMessage {
    pub timestamp: DateTime<Utc>,
    pub port_name: String,      // Display name (e.g., "GPS", "Motor")
    pub port_path: String,      // Actual path (e.g., "/dev/ttyUSB0")
    pub data: String,           // Line of data (without line ending)
}

// src/config/port_config.rs
#[derive(Deserialize, Serialize)]
pub struct PortConfig {
    pub name: String,                           // Required, unique
    pub path: String,                           // Required, unique (COM1, /dev/ttyUSB0)
    #[serde(default = "default_baud")]
    pub baud_rate: u32,                         // Default: 115200
    #[serde(default)]
    pub data_bits: DataBits,                    // Default: 8
    #[serde(default)]
    pub stop_bits: StopBits,                    // Default: 1
    #[serde(default)]
    pub parity: Parity,                         // Default: None
    #[serde(default)]
    pub flow_control: FlowControl,              // Default: None
    #[serde(default = "default_line_ending")]
    pub line_ending: String,                    // Default: "\n"
    #[serde(default = "random_color")]
    pub color: Color,                           // Default: random unique
}

// src/app.rs
pub struct AppState {
    pub ports: HashMap<String, PortState>,      // Keyed by port name
    pub display_buffer: VecDeque<SerialMessage>,
    pub notifications: VecDeque<Notification>,
    pub config: AppConfig,
    pub script_running: Option<ScriptHandle>,
    pub debug_logs: VecDeque<String>,
    pub mode: AppMode,                          // Normal, Debug, Command
}

// src/script/ast.rs
pub enum Expr {
    Number(f64),
    String(String),
    Bool(bool),
    Ident(String),
    Array(Vec<Expr>),
    BinaryOp { left: Box<Expr>, op: BinOp, right: Box<Expr> },
    UnaryOp { op: UnaryOp, expr: Box<Expr> },
    Call { name: String, args: Vec<Expr> },
    Index { array: Box<Expr>, index: Box<Expr> },
}

pub enum Stmt {
    Let { name: String, value: Expr },
    Assign { name: String, value: Expr },
    If { cond: Expr, then_block: Block, elif: Vec<(Expr, Block)>, else_block: Option<Block> },
    While { cond: Expr, body: Block },
    For { var: String, iter: Expr, body: Block },  // for i in 0..10 { }
    Fn { name: String, params: Vec<String>, body: Block },
    Return { value: Option<Expr> },
    Expr(Expr),  // Expression statement (e.g., sendstr(...);)
}
```

---

## Config File Format

```toml
# config/config.toml

[defaults]
baud_rate = 115200
data_bits = 8
stop_bits = 1
parity = "none"
flow_control = "none"
line_ending = "\n"

[[port]]
name = "GPS"
path = "/dev/ttyUSB0"
color = "#FF5733"

[[port]]
name = "Motor"
path = "/dev/ttyUSB1"
baud_rate = 9600          # Override default
line_ending = "\r\n"      # Override default

[[port]]
name = "Debug"
path = "/dev/ttyACM0"
# Uses all defaults, random color
```

```toml
# config/macros.toml

[[macro]]
name = "reset_all"
key = "F1"
command = "sendstr([\"GPS\", \"Motor\"], \"RESET\\n\")"

[[macro]]
name = "ping"
key = "F2"
command = "sendstr([\"GPS\"], \"PING\\n\")"
```

---

## Script Language (.stui)

Rust-like syntax with built-in serial commands.

### Types

- **Numbers**: `f64` (e.g., `3.14`, `42`)
- **Strings**: `"hello"` with escape sequences (`\n`, `\"`, `\\`)
- **Booleans**: `true`, `false`
- **Arrays**: `[1, 2, 3]`, `["COM1", "COM2"]`

### Built-in Functions

| Function | Description |
|----------|-------------|
| `sendstr(ports, string)` | Send string to ports |
| `sendbin(ports, hex)` | Send binary data (e.g., `"0x01020304"`) |
| `wait(seconds)` | Pause execution |
| `waitstr(ports, regex, timeout)` | Wait for regex match (aborts on timeout) |

### Example Script

```rust
// scripts/test_sequence.stui

fn setup_port(port) {
    sendstr([port], "INIT\n");
    wait(0.5);
    sendstr([port], "CONFIG MODE=AUTO\n");
    return true;
}

let ports = ["GPS", "Motor"];

for port in ports {
    if setup_port(port) {
        sendstr([port], "START\n");
    }
}

// Wait for acknowledgment
waitstr(["GPS"], r"OK|ACK", 5.0);

// Send binary data
sendbin(["Motor"], "0x01020304");

let counter = 0;
while counter < 10 {
    sendstr(["GPS"], "PING\n");
    wait(1.0);
    counter = counter + 1;
}
```

---

## UI Layout

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ [Port ▼] [Filter ▼] [Add Port] [Run Script ▼]              NOTIFICATIONS →  │
│  GPS                  Hide:                    scripts/                     │
│  Motor ✓              ☐ GPS                    └─ test.stui                 │
│  Debug                ☑ Debug                  └─ setup.stui                │
│  + Add Port                                                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│ [2025-12-07 14:32:01.123] <GPS> NMEA: $GPGGA,123456.00,4807.038,N...        │
│ [2025-12-07 14:32:01.456] <Motor> RPM: 1500, Temp: 45C                      │
│ [2025-12-07 14:32:01.789] <GPS> NMEA: $GPRMC,123456.00,A,4807.038...        │
│ [2025-12-07 14:32:02.012] <Motor> RPM: 1520, Temp: 46C                      │
│                                                                             │
│ ~ (vim navigation: j/k/gg/G/Ctrl-d/Ctrl-u, /search, y/yy)                  │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ PREVIEW (selected line, wrapped):                                           │
│ [2025-12-07 14:32:01.123] <GPS> NMEA: $GPGGA,123456.00,4807.038,N,01131.000 │
│ ,E,1,08,0.9,545.4,M,47.0,M,,*47                                             │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ ┌──────────┐                                                                │
│ │ ☑ GPS    │  :sendstr GPS "PING"                                          │
│ │ ☐ Motor  │  ───────────────────────────────────────────────────────────  │
│ │ ☑ Debug  │  (vim input: i=insert, Esc=normal, Ctrl+R=backsearch)         │
│ └──────────┘                                                                │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Commands

| Command | Description |
|---------|-------------|
| `:q` | Quit application |
| `:w` | Save config |
| `:debug` | Toggle debug screen |
| `:memory` | Show memory usage |
| `:thread` | Show task/thread status |
| `:run <script>` | Run a script |
| `:stop` | Abort running script |
| `:clear` | Clear display buffer |
| `:connect <port>` | Connect to port |
| `:disconnect <port>` | Disconnect from port |

---

## Vim Keybindings

### Display Navigation

| Key | Action |
|-----|--------|
| `j` / `k` | Scroll down/up by line |
| `Ctrl+d` / `Ctrl+u` | Half-page down/up |
| `gg` / `G` | Jump to top/bottom |
| `/` | Search (regex) |
| `n` / `N` | Next/previous match |
| `y` | Yank selection to clipboard |
| `yy` | Yank current line to clipboard |

### Input Box

| Key | Action |
|-----|--------|
| `i` | Enter insert mode |
| `Esc` | Enter normal mode |
| `Ctrl+R` | Reverse search (display buffer) |

---

## Dependencies

```toml
[dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }

# TUI
ratatui = { version = "0.29", features = ["all-widgets"] }
crossterm = "0.28"

# Serial
tokio-serial = "5.4"

# Config
toml = "0.8"
serde = { version = "1", features = ["derive"] }

# Time
chrono = "0.4"

# Error handling
color-eyre = "0.6"
thiserror = "2"

# Regex (same engine as ripgrep)
regex = "1"
grep-regex = "0.1"
grep-searcher = "0.1"

# Clipboard
arboard = "3"

# Random colors
rand = "0.8"

# Logging internals
tracing = "0.1"
tracing-subscriber = "0.3"
```

---

## Log Format

```
<2025-12-07 14:32:01.123>[GPS] NMEA: $GPGGA,123456.00,4807.038,N,01131.000,E,1,08,0.9,545.4,M,47.0,M,,*47
<2025-12-07 14:32:01.456>[Motor] RPM: 1500, Temp: 45C
```

Logs are written to:
- `logs/all.log` - Combined log from all ports
- `logs/<port_name>.log` - Individual per-port logs

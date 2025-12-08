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
│                              spawns all tasks                               │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                               APP STATE                                     │
│                            (Arc<Mutex<...>>)                                │
│                                                                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐            │
│  │PortConfigs  │ │DisplayBuffer│ │Notifications│ │ ScriptState │            │
│  │HashMap<name,│ │VecDeque<    │ │VecDeque<    │ │running flag,│            │
│  │PortConfig>  │ │SerialMsg>   │ │Notification>│ │abort handle │            │
│  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └──────┬──────┘            │
│         │               │               │               │                   │
└─────────┼───────────────┼───────────────┼───────────────┼───────────────────┘
          │               │               │               │
          │read           │read/push      │read/push      │read/write
          │               │               │               │
          │ UI Task       │ UI Task       │ UI Task       │ Script Engine
          │ Cmd Dispatcher│ Disp.Buf.Upd. │ Notif.System  │ Cmd Dispatcher
          ▼               ▼               ▼               ▼

╔═══════════════════════════════════════════════════════════════════════════════╗
║                                UI TASK                                        ║
║                               (ratatui)                                       ║
║                                                                               ║
║  READS FROM APP STATE:                                                        ║
║  - PortConfigs (to render config bar, port selector)                          ║
║  - DisplayBuffer (to render serial output)                                    ║
║  - Notifications (to render notification stack)                               ║
║  - ScriptState (to show "script running" indicator)                           ║
║                                                                               ║
║  ┌──────────────────────────────────────────────────────────────────────┐     ║
║  │                         RENDER LOOP                                  │     ║
║  │  1. Lock App State                                                   │     ║
║  │  2. Render config bar, display, preview, input box, notifications    │     ║
║  │  3. Unlock App State                                                 │     ║
║  │  4. Poll crossterm events                                            │     ║
║  └──────────────────────────────────────────────────────────────────────┘     ║
║                               │                                               ║
║                               │ crossterm::event::read()                      ║
║                               ▼                                               ║
║  ┌──────────────────────────────────────────────────────────────────────┐     ║
║  │                       INPUT HANDLER                                  │     ║
║  │                                                                      │     ║
║  │  Keyboard input is parsed into:                                      │     ║
║  │  - Vim navigation commands (j/k/gg/G/etc) -> handled locally         │     ║
║  │  - : commands (:q, :run, :debug) -> sent to Command Dispatcher       │     ║
║  │  - User text + Enter -> sent to Command Dispatcher                   │     ║
║  └───────────────────────────────┬──────────────────────────────────────┘     ║
╚══════════════════════════════════╪════════════════════════════════════════════╝
                                   │
                                   │ mpsc: AppCommand
                                   │ (Quit, RunScript, Debug, SendText, etc)
                                   ▼
╔══════════════════════════════════════════════════════════════════════════════╗
║                          COMMAND DISPATCHER                                  ║
║                            (async task)                                      ║
║                                                                              ║
║  Receives AppCommand and routes:                                             ║
║                                                                              ║
║  AppCommand::Quit           -> signals shutdown                              ║
║  AppCommand::RunScript(p)   -> spawns Script Engine task                     ║
║  AppCommand::StopScript     -> sends abort signal to Script Engine           ║
║  AppCommand::ToggleDebug    -> toggles debug_mode in App State               ║
║  AppCommand::SendText{..}   -> sends SerialCommand::Send to Serial Handler   ║
║  AppCommand::Connect{..}    -> sends SerialCommand::Connect                  ║
║  AppCommand::Disconnect{..} -> sends SerialCommand::Disconnect               ║
║  AppCommand::SaveConfig     -> saves config to file                          ║
║  AppCommand::ClearDisplay   -> clears display_buffer in App State            ║
╚═════════════════════════════════════════════════════════════════════════════╝
                              │
                              │ mpsc: SerialCommand
                              │ (Send, Connect, Disconnect)
                              │
        ┌─────────────────────┴─────────────────────┐
        │                                           │
        │ (from Command Dispatcher                  │ (from Script Engine
        │  when user types text)                    │  builtins: sendstr/sendbin)
        │                                           │
        ▼                                           │
╔═══════════════════════════════════════════════════╪═══════════════════════════╗
║                            SERIAL HANDLER         │                           ║
║                    (tokio tasks, one per port)    │                           ║
║                                                   │                           ║
║  ┌──────────────────────────────────────────┐     │                           ║
║  │ mpsc::Receiver<SerialCommand>            │◄────┘                           ║
║  │                                          │                                 ║
║  │ SerialCommand::Send { ports, data }      │                                 ║
║  │   -> routes data to specified port tasks │                                 ║
║  │ SerialCommand::Connect { port }          │                                 ║
║  │   -> spawns new port task                │                                 ║
║  │ SerialCommand::Disconnect { port }       │                                 ║
║  │   -> kills port task                     │                                 ║
║  └──────────────────────────────────────────┘                                 ║
║                                                                               ║
║  ┌─────────┐  ┌─────────┐  ┌─────────┐            ┌─────────┐                 ║
║  │  COM1   │  │  COM2   │  │  COM3   │    ...     │ COM255  │                 ║
║  │  task   │  │  task   │  │  task   │            │  task   │                 ║
║  │         │  │         │  │         │            │         │                 ║
║  │ ┌─────┐ │  │ ┌─────┐ │  │ ┌─────┐ │            │ ┌─────┐ │                 ║
║  │ │read │ │  │ │read │ │  │ │read │ │            │ │read │ │                 ║
║  │ │  ↓  │ │  │ │  ↓  │ │  │ │  ↓  │ │            │ │  ↓  │ │                 ║
║  │ │write│ │  │ │write│ │  │ │write│ │            │ │write│ │                 ║
║  │ └─────┘ │  │ └─────┘ │  │ └─────┘ │            │ └─────┘ │                 ║
║  └────┬────┘  └────┬────┘  └────┬────┘            └────┬────┘                 ║
║       │            │            │                      │                      ║
║       │ Each port task reads lines, creates SerialMessage:                    ║
║       │ SerialMessage { timestamp, port_name, port_path, data }               ║
║       └────────────┴────────────┴──────────────────────┘                      ║
║                              │                                                ║
║                    broadcast::Sender<SerialMessage>                           ║
║                      (fan-out to all subscribers)                             ║
╚══════════════════════════════╤════════════════════════════════════════════════╝
                               │
                               │ broadcast::Receiver<SerialMessage>
                               │ (each subscriber clones receiver, gets all msgs)
                               │
        ┌──────────────────────┼──────────────────────┐
        │                      │                      │
        ▼                      ▼                      ▼
╔══════════════════╗  ╔══════════════════╗  ╔══════════════════╗
║ DISPLAY BUFFER   ║  ║     LOGGER       ║  ║  SCRIPT ENGINE   ║
║    UPDATER       ║  ║    (async)       ║  ║    (async)       ║
║                  ║  ║                  ║  ║                  ║
║ Receives:        ║  ║ Receives:        ║  ║ waitstr builtin: ║
║ SerialMessage    ║  ║ SerialMessage    ║  ║ subscribes to    ║
║                  ║  ║                  ║  ║ broadcast, waits ║
║ Action:          ║  ║ Action:          ║  ║ for regex match  ║
║ Push to App      ║  ║ Format as:       ║  ║ on specified     ║
║ State's          ║  ║ <ts>[port] data  ║  ║ ports with       ║
║ DisplayBuffer    ║  ║                  ║  ║ timeout          ║
║                  ║  ║ Write to:        ║  ║                  ║
╚════════╤═════════╝  ║ - logs/all.log   ║  ║ On match:        ║
         │            ║ - logs/<port>.log║  ║ -> continue      ║
         │            ╚════════╤═════════╝  ║ On timeout:      ║
         │                     │            ║ -> abort script  ║
         │ push                │ write      ╚════════╤═════════╝
         ▼                     ▼                     │
┌──────────────┐       ┌──────────────┐              │
│ App State:   │       │  Log Files   │              │ sendstr/sendbin
│ DisplayBuffer│       │              │              │ builtins send
└──────────────┘       │  logs/       │              │ SerialCommand
                       │  ├─ all.log  │              │
                       │  ├─ GPS.log  │              ▼
                       │  └─ Motor.log│      ┌───────────────┐
                       └──────────────┘      │mpsc::Sender   │
                                             │<SerialCommand>│
                                             │               │
                                             │ (loops back   │
                                             │  to Serial    │
                                             │  Handler)     │
                                             └───────────────┘

╔═══════════════════════════════════════════════════════════════════════════════╗
║                          NOTIFICATION SYSTEM                                  ║
║                            (async task)                                       ║
║                                                                               ║
║  ┌─────────────────────────────────────────────────────────────────────────┐  ║
║  │ mpsc::Receiver<Notification>                                            │  ║
║  │                                                                         │  ║
║  │ Any component can send notifications:                                   │  ║
║  │ - Serial Handler: "Connected to GPS", "Disconnected from Motor"         │  ║
║  │ - Script Engine: "Script started", "Script finished", "Script aborted"  │  ║
║  │ - Config Manager: "Config error at line 5"                              │  ║
║  │ - Command Dispatcher: "Unknown command: :foo"                           │  ║
║  └─────────────────────────────────────────────────────────────────────────┘  ║
║                                      │                                        ║
║                                      │ push                                   ║
║                                      ▼                                        ║
║  ┌─────────────────────────────────────────────────────────────────────────┐  ║
║  │ App State: Notifications (VecDeque<Notification>)                       │  ║
║  │                                                                         │  ║
║  │ UI Task reads this to render notification stack at top-right            │  ║
║  │ Each notification has:                                                  │  ║
║  │ - message: String                                                       │  ║
║  │ - duration: calculated from char_count / reading_speed + 1 sec          │  ║
║  │ - created_at: Instant (for auto-dismiss)                                │  ║
║  └─────────────────────────────────────────────────────────────────────────┘  ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### Channel Architecture

```rust
// Channels for communication between components

// Input Handler -> Command Dispatcher
app_command_tx: tokio::sync::mpsc::Sender<AppCommand>

pub enum AppCommand {
    Quit,
    RunScript { path: PathBuf },
    StopScript,
    ToggleDebug,
    SendText { ports: Vec<String>, text: String },
    Connect { port_name: String },
    Disconnect { port_name: String },
    SaveConfig,
    ClearDisplay,
}

// Command Dispatcher / Script Engine -> Serial Handler
serial_command_tx: tokio::sync::mpsc::Sender<SerialCommand>

pub enum SerialCommand {
    Send { port_names: Vec<String>, data: Vec<u8> },
    Connect { port_name: String },
    Disconnect { port_name: String },
}

// Serial Handler -> all subscribers (Display Buffer Updater, Logger, Script Engine waitstr)
serial_broadcast: tokio::sync::broadcast::Sender<SerialMessage>

// Any component -> Notification System
notification_tx: tokio::sync::mpsc::Sender<Notification>

pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,  // Info, Warning, Error
    pub created_at: Instant,
}

// Command Dispatcher -> Script Engine (for abort signal)
script_abort_tx: tokio::sync::oneshot::Sender<()>
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
    pub port_configs: HashMap<String, PortConfig>,  // Keyed by port name
    pub port_states: HashMap<String, PortState>,    // Runtime state (connected, etc)
    pub display_buffer: VecDeque<SerialMessage>,
    pub notifications: VecDeque<Notification>,
    pub script_running: bool,
    pub script_abort_handle: Option<oneshot::Sender<()>>,
    pub debug_mode: bool,
    pub debug_logs: VecDeque<String>,
    pub vim_mode: VimMode,                          // Normal, Insert, Command, Search
    pub focus: Focus,                               // Display, InputBox
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
│ <2025-12-07 14:32:01.123>[GPS] NMEA: $GPGGA,123456.00,4807.038,N...         │
│ <2025-12-07 14:32:01.456>[Motor] RPM: 1500, Temp: 45C                       │
│ <2025-12-07 14:32:01.789>[GPS] NMEA: $GPRMC,123456.00,A,4807.038...         │
│ <2025-12-07 14:32:02.012>[Motor] RPM: 1520, Temp: 46C                       │
│                                                                             │
│ ~ (vim navigation: j/k/gg/G/Ctrl-d/Ctrl-u, /search, y/yy)                  │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ PREVIEW (selected line, wrapped):                                           │
│ <2025-12-07 14:32:01.123>[GPS] NMEA: $GPGGA,123456.00,4807.038,N,01131.000, │
│ E,1,08,0.9,545.4,M,47.0,M,,*47                                              │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ ┌──────────┐                                                                │
│ │ ☑ GPS    │  PING█                                                        │
│ │ ☐ Motor  │  ───────────────────────────────────────────────────────────  │
│ │ ☑ Debug  │  (vim input: i=insert, Esc=normal, Ctrl+R=backsearch)         │
│ └──────────┘  Press Enter to send to selected ports                        │
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

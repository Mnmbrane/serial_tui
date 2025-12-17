# SerialTUI Implementation Guide

A step-by-step guide to building SerialTUI from scratch, with explanations for every design decision.

---

## Table of Contents

1. [Overview](#overview)
2. [Phase 1: Project Foundation](#phase-1-project-foundation)
3. [Phase 2: Config Module](#phase-2-config-module)
4. [Phase 3: Serial Module](#phase-3-serial-module-single-port)
5. [Phase 4: Basic TUI](#phase-4-basic-tui)
6. [Phase 5: Channel Wiring](#phase-5-channel-wiring)
7. [Phase 6: Multi-Port Support](#phase-6-multi-port-support)
8. [Phase 7: Vim Navigation](#phase-7-vim-navigation)
9. [Phase 8: Notifications](#phase-8-notifications)
10. [Phase 9: Logger Module](#phase-9-logger-module)
11. [Phase 10: Script Engine](#phase-10-script-engine)

---

## Overview

### What We're Building

SerialTUI is a terminal UI for serial port communication. Think of it as a powerful replacement for tools like `teraterm`, or PuTTY's serial console, but with:

- Multiple ports at once (up to 255)
- Vim-style keyboard navigation
- Scriptable automation
- Modern async architecture

### Why This Architecture?

```
┌─────────────┐     ┌──────────────────┐     ┌───────────────┐
│   UI Task   │────▶│ Command          │────▶│ Serial        │
│  (ratatui)  │     │ Dispatcher       │     │ Handler       │
└─────────────┘     └──────────────────┘     └───────┬───────┘
      ▲                                              │
      │                                     broadcast channel
      │                                              │
      └──────────────────────────────────────────────┘
                    (serial messages)
```

**Why separate tasks connected by channels?**

1. **Responsiveness**: The UI never blocks waiting for serial data. If a port is slow or disconnected, the UI stays snappy.

2. **Scalability**: Each serial port runs in its own task. Adding more ports doesn't slow down existing ones.

3. **Testability**: Components can be tested in isolation. You can test the UI without real serial ports.

4. **Crash isolation**: If one port task panics, others keep running.

**Why broadcast channel for serial data?**

Multiple consumers need the same data:

- Display buffer (show on screen)
- Logger (write to files)
- Script engine (waitstr matching)

A broadcast channel lets all of them receive every message without coordination.

---

## Phase 1: Project Foundation

**Goal:** Get a compiling project with proper error handling infrastructure.

### Step 1.1: Dependencies

```toml
[package]
name = "serial_tui"
version = "0.1.0"
edition = "2024"

[dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }

# TUI
ratatui = "0.29"
crossterm = "0.28"

# Serial
tokio-serial = "5.4"

# Config
toml = "0.8"
serde = { version = "1", features = ["derive"] }

# Time
chrono = { version = "0.4", features = ["serde"] }

# Error handling
color-eyre = "0.6"
thiserror = "2"

# Regex
regex = "1"

# Clipboard
arboard = "3"

# Random colors
rand = "0.8"

# Logging internals
tracing = "0.1"
tracing-subscriber = "0.3"
```

#### Why These Specific Crates?

| Crate | Why This One? | Alternatives Considered |
|-------|---------------|------------------------|
| **tokio** | Industry standard async runtime. "full" features gives us everything (timers, fs, sync primitives). Most other async crates are built for tokio. | async-std (less ecosystem support), smol (smaller but less mature) |
| **ratatui** | Active fork of tui-rs with better maintenance. Large widget library, good docs. | tui-rs (abandoned), cursive (different paradigm), termion (lower level) |
| **crossterm** | Cross-platform terminal manipulation. Works on Windows, Linux, macOS. | termion (Linux only), ncurses (C dependency) |
| **tokio-serial** | Async serial port that integrates with tokio. Uses tokio's `AsyncRead`/`AsyncWrite`. | serialport (sync only, would need spawn_blocking) |
| **serde + toml** | TOML is human-readable and git-friendly. Serde makes parsing trivial with derive macros. | JSON (harder to edit by hand), YAML (indentation issues), RON (less familiar) |
| **chrono** | Full-featured datetime library. Needed for timestamps with millisecond precision. | time (less features), std::time (no formatting) |
| **color-eyre** | Beautiful error reports with backtraces. Great for debugging during development. | anyhow (less pretty), plain Result (no context) |
| **thiserror** | Derive macro for custom error types. Works well with color-eyre. | manual Error impl (boilerplate), anyhow (no typed errors) |
| **regex** | Same engine as ripgrep. Fast, Unicode-aware, well-tested. | fancy-regex (slower), pcre2 (C dependency) |
| **arboard** | Cross-platform clipboard. Simple API. | clipboard (older), copypasta (less maintained) |

#### Why Edition 2024?

Rust 2024 edition includes:

- Improved async ergonomics
- Better error messages
- New language features

If you have issues, fall back to `edition = "2021"`.

---

### Step 1.2: Directory Structure

```
src/
├── main.rs              # Entry point, wires everything together
├── app.rs               # AppState struct, shared state
├── error.rs             # Custom error types
│
├── config/
│   ├── mod.rs           # Re-exports
│   ├── port_config.rs   # PortConfig struct
│   └── manager.rs       # Load/save config files
│
├── serial/
│   ├── mod.rs           # Re-exports
│   ├── message.rs       # SerialMessage struct
│   ├── command.rs       # SerialCommand enum
│   ├── handler.rs       # Port manager (spawns port tasks)
│   └── port.rs          # Single port read/write task
│
├── ui/
│   ├── mod.rs           # Re-exports
│   ├── app_ui.rs        # Main render function
│   ├── display.rs       # Serial output widget
│   ├── input_box.rs     # Text input widget
│   └── vim/
│       ├── mod.rs
│       └── mode.rs      # VimMode enum, Focus enum
│
├── logger/
│   ├── mod.rs
│   └── writer.rs        # Async log file writer
│
└── notification/
    ├── mod.rs
    └── system.rs        # Notification queue
```

#### Why This Structure?

**Separation by domain, not by type.**

Bad structure (by type):

```
src/
├── structs/
├── enums/
├── traits/
└── functions/
```

Good structure (by domain):

```
src/
├── serial/      # Everything about serial ports
├── config/      # Everything about configuration
├── ui/          # Everything about rendering
```

**Why?**

1. **Locality**: When working on serial ports, all relevant code is in one directory.
2. **Encapsulation**: Each module has a clear public API via `mod.rs`.
3. **Compile times**: Rust compiles modules in parallel. Isolated modules = faster incremental builds.

**Why `mod.rs` files?**

The `mod.rs` pattern lets you control what's public:

```rust
// src/serial/mod.rs
mod command;    // Private implementation
mod message;    // Private implementation
mod port;       // Private implementation

pub use command::SerialCommand;  // Public API
pub use message::SerialMessage;  // Public API
pub use port::run_port_task;     // Public API
```

External code sees only what you export. Internal details stay hidden.

---

### Step 1.3: Error Types

Create `src/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Config error: {0}")]
    Config(String),

    #[error("Serial error: {0}")]
    Serial(#[from] tokio_serial::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Channel closed")]
    ChannelClosed,
}

pub type Result<T> = std::result::Result<T, AppError>;
```

#### Why Custom Error Types?

**Problem with string errors:**

```rust
fn load_config() -> Result<Config, String> {
    // Caller can't distinguish between "file not found" and "parse error"
    Err("something went wrong".to_string())
}
```

**Problem with `Box<dyn Error>`:**

```rust
fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    // Caller can't pattern match on error type
    // No compile-time guarantees about what errors are possible
}
```

**Why enum errors are better:**

```rust
fn load_config() -> Result<Config, AppError> {
    // Caller knows exactly what can fail
    // Can pattern match: if let AppError::Config(msg) = err { ... }
    // Compiler warns about unhandled variants
}
```

#### Why `#[from]` Attribute?

The `#[from]` attribute auto-implements `From<T>`:

```rust
#[error("IO error: {0}")]
Io(#[from] std::io::Error),
```

This lets you use `?` operator:

```rust
let content = std::fs::read_to_string(path)?;  // io::Error auto-converts to AppError::Io
```

Without `#[from]`, you'd need:

```rust
let content = std::fs::read_to_string(path).map_err(AppError::Io)?;
```

#### Why a Type Alias for Result?

```rust
pub type Result<T> = std::result::Result<T, AppError>;
```

Now you can write:

```rust
fn load_config() -> Result<Config>  // Instead of Result<Config, AppError>
```

Less typing, clearer intent.

---

### Step 1.4: Basic Main

Create `src/main.rs`:

```rust
mod app;
mod config;
mod error;
mod logger;
mod notification;
mod serial;
mod ui;

use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    println!("SerialTUI starting...");

    Ok(())
}
```

#### Why `#[tokio::main]`?

This macro transforms:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // async code
}
```

Into:

```rust
fn main() -> Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            // async code
        })
}
```

It creates the tokio runtime for you. The `multi_thread` variant uses all CPU cores.

#### Why `color_eyre::install()`?

This sets up panic and error hooks that:

1. Print colorized error messages
2. Include backtraces
3. Show source code context

Call it once at startup, before any errors can occur.

**Checkpoint:** Run `cargo build`. If it compiles, Phase 1 is done.

---

## Phase 2: Config Module

**Goal:** Load and save port configuration from TOML files.

### Step 2.1: PortConfig Struct

Create `src/config/port_config.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConfig {
    /// Display name shown in UI (e.g., "GPS", "Motor")
    pub name: String,

    /// System path to serial port (e.g., "/dev/ttyUSB0", "COM3")
    pub path: String,

    /// Baud rate. Common values: 9600, 115200, 921600
    #[serde(default = "default_baud")]
    pub baud_rate: u32,

    /// Characters appended when sending data
    #[serde(default = "default_line_ending")]
    pub line_ending: String,

    /// Hex color for display (e.g., "#FF5733")
    #[serde(default = "random_color")]
    pub color: String,
}

fn default_baud() -> u32 { 115200 }
fn default_line_ending() -> String { "\n".to_string() }
fn random_color() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("#{:02X}{:02X}{:02X}",
        rng.gen_range(100..255),  // Avoid dark colors (< 100)
        rng.gen_range(100..255),
        rng.gen_range(100..255))
}
```

#### Why Serde Derive?

The `#[derive(Serialize, Deserialize)]` macro generates code to convert between Rust structs and various formats (TOML, JSON, etc.):

```rust
// Deserialize: TOML string → Rust struct
let config: PortConfig = toml::from_str(toml_string)?;

// Serialize: Rust struct → TOML string
let toml_string = toml::to_string(&config)?;
```

Without serde, you'd write hundreds of lines of parsing code.

#### Why `#[serde(default = "function")]`?

This makes fields optional in the config file:

```toml
[[port]]
name = "GPS"
path = "/dev/ttyUSB0"
# baud_rate, line_ending, color are all optional
# They'll use default_baud(), default_line_ending(), random_color()
```

The user only specifies what they want to customize.

#### Why Random Colors Start at 100?

```rust
rng.gen_range(100..255)  // Not 0..255
```

Colors below RGB(100, 100, 100) are too dark to see on typical dark terminal backgrounds. Starting at 100 ensures readable colors.

---

### Step 2.2: Config Manager

Create `src/config/manager.rs`:

```rust
use super::port_config::PortConfig;
use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub port: Vec<PortConfig>,
}

impl Config {
    /// Load config from file. Returns empty config if file doesn't exist.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            // No config file = empty config (not an error)
            // User might be running for the first time
            return Ok(Config { port: vec![] });
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| AppError::Config(format!("Failed to read {}: {}", path.display(), e)))?;

        toml::from_str(&content)
            .map_err(|e| AppError::Config(format!("Failed to parse {}: {}", path.display(), e)))
    }

    /// Save config to file. Creates parent directories if needed.
    pub fn save(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| AppError::Config(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate config. Returns list of errors (empty = valid).
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Check for duplicate names
        let mut names = std::collections::HashSet::new();
        for port in &self.port {
            if !names.insert(&port.name) {
                errors.push(format!("Duplicate port name: {}", port.name));
            }
        }

        // Check for duplicate paths
        let mut paths = std::collections::HashSet::new();
        for port in &self.port {
            if !paths.insert(&port.path) {
                errors.push(format!("Duplicate port path: {}", port.path));
            }
        }

        errors
    }
}
```

#### Why Return Empty Config Instead of Error?

```rust
if !path.exists() {
    return Ok(Config { port: vec![] });  // Not Err(...)
}
```

**First-run experience matters.**

If a new user runs the app and gets "Error: config.toml not found", they'll think something is broken. Returning an empty config lets them:

1. See the UI immediately
2. Add ports through the UI
3. Save config when ready

#### Why Validate Separately?

```rust
pub fn validate(&self) -> Vec<String>  // Returns errors, not Result
```

Validation returns a list of problems, not a single error. This lets you show all issues at once:

```
Config errors:
- Duplicate port name: GPS
- Duplicate port path: /dev/ttyUSB0
- Invalid baud rate: -1
```

Instead of fixing one error, rerunning, finding the next error, repeat.

#### Why `to_string_pretty`?

```rust
toml::to_string_pretty(self)  // Not to_string()
```

`to_string_pretty` adds newlines and indentation:

```toml
# Pretty (readable)
[[port]]
name = "GPS"
path = "/dev/ttyUSB0"
baud_rate = 115200

# Not pretty (one long line)
port = [{name = "GPS", path = "/dev/ttyUSB0", baud_rate = 115200}]
```

Config files are edited by humans. Make them human-friendly.

---

### Step 2.3: Config mod.rs

Create `src/config/mod.rs`:

```rust
mod manager;
mod port_config;

pub use manager::Config;
pub use port_config::PortConfig;
```

#### Why Re-export?

Without re-exports, external code needs:

```rust
use crate::config::manager::Config;
use crate::config::port_config::PortConfig;
```

With re-exports:

```rust
use crate::config::{Config, PortConfig};
```

The internal file structure is hidden. You can reorganize files without breaking external code.

**Checkpoint:** Create `config/config.toml`:

```toml
[[port]]
name = "Test"
path = "/dev/ttyUSB0"
```

Add to main.rs:

```rust
let config = config::Config::load(Path::new("config/config.toml"))?;
println!("Loaded {} ports", config.port.len());
```

---

## Phase 3: Serial Module (Single Port)

**Goal:** Read from one serial port asynchronously.

### Step 3.1: SerialMessage

Create `src/serial/message.rs`:

```rust
use chrono::{DateTime, Utc};

/// A single line of data received from a serial port.
#[derive(Debug, Clone)]
pub struct SerialMessage {
    /// When the message was received (UTC)
    pub timestamp: DateTime<Utc>,

    /// Human-readable port name (e.g., "GPS")
    pub port_name: String,

    /// System path (e.g., "/dev/ttyUSB0")
    pub port_path: String,

    /// The actual data (one line, without line ending)
    pub data: String,
}
```

#### Why Both `port_name` and `port_path`?

**port_name** is for humans:

- Shown in UI: `[GPS] NMEA: $GPGGA...`
- Used in log filenames: `GPS.log`
- Used in scripts: `sendstr(["GPS"], "PING")`

**port_path** is for the system:

- Actual device path: `/dev/ttyUSB0`
- Needed if you want to reconnect
- Useful for debugging ("which physical port is GPS?")

Keeping both avoids repeated lookups.

#### Why UTC Timestamps?

```rust
pub timestamp: DateTime<Utc>,  // Not DateTime<Local>
```

1. **Consistency**: All messages use the same timezone, even if system timezone changes.
2. **Logging**: Log files are unambiguous across timezones.
3. **Sorting**: UTC timestamps sort correctly without timezone conversion.

Display can convert to local time when rendering.

#### Why `Clone`?

```rust
#[derive(Debug, Clone)]
```

Messages are sent through a broadcast channel. Each receiver gets a clone. Without `Clone`, you'd need `Arc<SerialMessage>` everywhere.

For small structs like this, cloning is cheap. For large data, you'd use `Arc` or `Bytes`.

---

### Step 3.2: SerialCommand

Create `src/serial/command.rs`:

```rust
/// Commands sent to the serial handler to control ports.
#[derive(Debug, Clone)]
pub enum SerialCommand {
    /// Send data to one or more ports
    Send {
        /// Port names to send to (e.g., ["GPS", "Motor"])
        port_names: Vec<String>,
        /// Raw bytes to send
        data: Vec<u8>
    },

    /// Connect to a port (start reading)
    Connect {
        port_name: String
    },

    /// Disconnect from a port (stop reading)
    Disconnect {
        port_name: String
    },
}
```

#### Why an Enum for Commands?

**Alternative: Multiple channels**

```rust
let (send_tx, send_rx) = mpsc::channel();      // For send commands
let (connect_tx, connect_rx) = mpsc::channel(); // For connect commands
let (disconnect_tx, disconnect_rx) = mpsc::channel(); // For disconnect commands
```

Problems:

- Three channels to manage
- Handler needs `select!` over three receivers
- Adding new commands means new channels

**Better: Single channel with enum**

```rust
let (cmd_tx, cmd_rx) = mpsc::channel::<SerialCommand>();
```

Benefits:

- One channel for all commands
- Adding commands = adding enum variants
- Pattern matching handles dispatch

#### Why `Vec<u8>` for Data?

```rust
data: Vec<u8>  // Not String
```

Serial data can be:

- Text: `"PING\r\n"`
- Binary: `[0x01, 0x02, 0x03, 0x04]`
- Mixed: `"DATA:" + [binary payload]`

`Vec<u8>` handles all cases. `String` would reject invalid UTF-8.

The `sendstr` script function converts strings to bytes. The `sendbin` function parses hex strings to bytes. Both produce `Vec<u8>`.

---

### Step 3.3: Single Port Task

Create `src/serial/port.rs`:

```rust
use super::message::SerialMessage;
use crate::config::PortConfig;
use chrono::Utc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::broadcast;
use tokio_serial::SerialPortBuilderExt;

/// Run a task that reads from a serial port and broadcasts messages.
///
/// This function loops forever, reconnecting on errors.
/// To stop it, abort the task handle.
pub async fn run_port_task(
    config: PortConfig,
    tx: broadcast::Sender<SerialMessage>,
) {
    let mut reconnect_delay = std::time::Duration::from_secs(1);
    const MAX_DELAY: std::time::Duration = std::time::Duration::from_secs(8);

    loop {
        match connect_and_read(&config, &tx).await {
            Ok(()) => {
                // Clean shutdown (task was aborted)
                break;
            }
            Err(e) => {
                eprintln!("[{}] Error: {}. Reconnecting in {:?}...",
                    config.name, e, reconnect_delay);

                tokio::time::sleep(reconnect_delay).await;

                // Exponential backoff: 1s -> 2s -> 4s -> 8s -> 8s -> ...
                reconnect_delay = std::cmp::min(reconnect_delay * 2, MAX_DELAY);
            }
        }
    }
}

async fn connect_and_read(
    config: &PortConfig,
    tx: &broadcast::Sender<SerialMessage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Open serial port
    let port = tokio_serial::new(&config.path, config.baud_rate)
        .open_native_async()?;

    // Reset reconnect delay on successful connection
    // (handled by caller resetting delay after Ok)

    // Wrap in buffered reader for line-by-line reading
    let mut reader = BufReader::new(port);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;

        if bytes_read == 0 {
            // EOF = port disconnected
            return Err("Port disconnected (EOF)".into());
        }

        let msg = SerialMessage {
            timestamp: Utc::now(),
            port_name: config.name.clone(),
            port_path: config.path.clone(),
            data: line.trim_end().to_string(),
        };

        // Send to broadcast channel
        // Ignore error (happens if no receivers, which is fine)
        let _ = tx.send(msg);
    }
}
```

#### Why Exponential Backoff?

When a port disconnects, you want to reconnect. But not too aggressively:

**Without backoff (immediate retry):**

```
[GPS] Error: Device not found
[GPS] Error: Device not found
[GPS] Error: Device not found
... (thousands of times per second)
```

This wastes CPU and floods logs.

**With exponential backoff:**

```
[GPS] Error: Device not found. Reconnecting in 1s...
[GPS] Error: Device not found. Reconnecting in 2s...
[GPS] Error: Device not found. Reconnecting in 4s...
[GPS] Error: Device not found. Reconnecting in 8s...
[GPS] Error: Device not found. Reconnecting in 8s...  (capped)
```

The delay doubles each time (1→2→4→8) but caps at 8 seconds. When the device comes back, you reconnect within 8 seconds.

#### Why `BufReader`?

```rust
let mut reader = BufReader::new(port);
```

Serial data arrives byte-by-byte. Without buffering:

```rust
// Bad: One syscall per byte
let mut byte = [0u8; 1];
port.read(&mut byte).await?;
```

With `BufReader`:

```rust
// Good: Reads chunks, returns lines
reader.read_line(&mut line).await?;
```

`BufReader` reads large chunks into an internal buffer, then returns data from the buffer. Far fewer syscalls.

#### Why `read_line` Returns Line Without Newline?

Actually, `read_line` *includes* the newline. That's why we do:

```rust
data: line.trim_end().to_string(),
```

`trim_end()` removes trailing `\n`, `\r\n`, or any whitespace. This normalizes line endings across platforms.

#### Why Ignore Broadcast Send Errors?

```rust
let _ = tx.send(msg);  // Ignore result
```

`broadcast::Sender::send` fails if there are no receivers. This is fine:

- At startup, receivers might not exist yet
- If all receivers disconnect, we still want to keep reading

The `let _ =` explicitly ignores the result (Rust warns about unused Results).

---

### Step 3.4: Serial mod.rs

Create `src/serial/mod.rs`:

```rust
mod command;
mod message;
mod port;

pub use command::SerialCommand;
pub use message::SerialMessage;
pub use port::run_port_task;
```

**Checkpoint:** Test with a real serial port or a virtual one:

```rust
// In main.rs
let (tx, mut rx) = broadcast::channel(1000);

let config = PortConfig {
    name: "Test".to_string(),
    path: "/dev/ttyUSB0".to_string(),  // Adjust for your system
    baud_rate: 115200,
    line_ending: "\n".to_string(),
    color: "#FF0000".to_string(),
};

tokio::spawn(run_port_task(config, tx));

while let Ok(msg) = rx.recv().await {
    println!("[{}] {}", msg.port_name, msg.data);
}
```

---

## Phase 4: Basic TUI

**Goal:** Display serial messages in a terminal interface.

### Step 4.1: AppState

Create `src/app.rs`:

```rust
use crate::config::PortConfig;
use crate::serial::SerialMessage;
use std::collections::{HashMap, VecDeque};
use tokio::sync::Mutex;
use std::sync::Arc;

/// Central application state shared between all tasks.
pub struct AppState {
    /// Port configurations keyed by name
    pub port_configs: HashMap<String, PortConfig>,

    /// Buffer of received messages (newest at back)
    pub display_buffer: VecDeque<SerialMessage>,

    /// Scroll position (0 = bottom/newest, higher = older)
    pub scroll_offset: usize,

    /// Current text in input box
    pub input_text: String,

    /// Whether app should keep running
    pub running: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            port_configs: HashMap::new(),
            display_buffer: VecDeque::with_capacity(10000),
            scroll_offset: 0,
            input_text: String::new(),
            running: true,
        }
    }
}

/// Thread-safe handle to AppState
pub type SharedState = Arc<Mutex<AppState>>;
```

#### Why `Arc<Mutex<T>>`?

Multiple async tasks need to access `AppState`:

- UI task reads it to render
- Display buffer updater writes new messages
- Command dispatcher modifies settings

**`Arc`** (Atomic Reference Counted) allows multiple owners:

```rust
let state = Arc::new(Mutex::new(AppState::new()));
let state_clone = state.clone();  // Both point to same data
```

**`Mutex`** ensures only one task accesses data at a time:

```rust
let mut guard = state.lock().await;  // Waits if another task has the lock
guard.display_buffer.push_back(msg);
// Lock released when guard is dropped
```

#### Why `tokio::sync::Mutex` Instead of `std::sync::Mutex`?

```rust
use tokio::sync::Mutex;  // Not std::sync::Mutex
```

**`std::sync::Mutex`** blocks the entire thread while waiting.
**`tokio::sync::Mutex`** yields to the async runtime while waiting.

In async code, blocking a thread is bad—it can't run other tasks. Tokio's mutex cooperates with the scheduler.

**Exception:** For very short critical sections with no `.await` inside, `std::sync::Mutex` is faster. But `tokio::sync::Mutex` is safer by default.

#### Why `VecDeque` for Display Buffer?

```rust
pub display_buffer: VecDeque<SerialMessage>,
```

`VecDeque` is a double-ended queue. It's efficient for:

- `push_back`: Add new messages (O(1))
- `pop_front`: Remove old messages when buffer is full (O(1))
- Iteration: Render visible portion

`Vec` would be O(n) for `pop_front` (shifts all elements).

#### Why Pre-allocate with Capacity?

```rust
VecDeque::with_capacity(10000)
```

Without capacity, the deque starts empty and grows by reallocating. With capacity, it pre-allocates space for 10,000 messages.

This avoids allocations during normal operation. Memory is cheap; stuttering UI is annoying.

---

### Step 4.2: Main Layout

Create `src/ui/app_ui.rs`:

```rust
use crate::app::AppState;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the entire UI.
pub fn render(frame: &mut Frame, state: &AppState) {
    // Split screen into two areas: display (flexible) and input (fixed height)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // Display takes remaining space (min 3 lines)
            Constraint::Length(3), // Input is exactly 3 lines
        ])
        .split(frame.area());

    render_display(frame, state, chunks[0]);
    render_input(frame, state, chunks[1]);
}

fn render_display(frame: &mut Frame, state: &AppState, area: ratatui::layout::Rect) {
    // Calculate visible range
    let height = area.height as usize - 2; // Subtract 2 for borders
    let total = state.display_buffer.len();

    // scroll_offset=0 means show newest (bottom)
    // scroll_offset=10 means skip 10 newest, show older
    let start = total.saturating_sub(height + state.scroll_offset);
    let end = total.saturating_sub(state.scroll_offset);

    let lines: Vec<Line> = state.display_buffer
        .iter()
        .skip(start)
        .take(end - start)
        .map(|msg| {
            Line::from(vec![
                Span::styled(
                    format!("[{}] ", msg.port_name),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(&msg.data),
            ])
        })
        .collect();

    let display = Paragraph::new(lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Serial Output"));

    frame.render_widget(display, area);
}

fn render_input(frame: &mut Frame, state: &AppState, area: ratatui::layout::Rect) {
    let input = Paragraph::new(state.input_text.as_str())
        .block(Block::default()
            .borders(Borders::ALL)
            .title("Input"));

    frame.render_widget(input, area);
}
```

#### Why This Layout?

```
┌─────────────────────────────────┐
│ Serial Output                   │  <- Flexible height (Constraint::Min)
│ [GPS] data...                   │
│ [Motor] data...                 │
│                                 │
├─────────────────────────────────┤
│ Input                           │  <- Fixed height (Constraint::Length)
│ > user typing here              │
└─────────────────────────────────┘
```

**`Constraint::Min(3)`** means "at least 3 lines, but take as much as available."
**`Constraint::Length(3)`** means "exactly 3 lines."

The display area grows/shrinks with terminal size. The input stays constant.

#### Why Scroll Offset Works This Way?

```rust
let start = total.saturating_sub(height + state.scroll_offset);
let end = total.saturating_sub(state.scroll_offset);
```

Think of `scroll_offset` as "how many lines from the bottom are hidden":

- `scroll_offset=0`: Show the newest `height` lines (normal view)
- `scroll_offset=10`: Hide the 10 newest lines, show older ones

This is inverted from typical scrolling but matches vim's `Ctrl+E`/`Ctrl+Y` behavior.

`saturating_sub` prevents underflow (returns 0 instead of panicking on negative).

#### Why Ratatui's Immediate Mode?

Ratatui uses immediate mode rendering:

```rust
loop {
    terminal.draw(|frame| render(frame, &state))?;  // Redraw everything
    handle_input()?;
}
```

Every frame, you redraw the entire UI from state. There's no "update this widget" API.

**Benefits:**

- Simple mental model (state → UI)
- No stale widgets
- Easy to reason about

**Tradeoff:**

- Redraws everything even if nothing changed
- Fast enough for TUIs (terminals are slow anyway)

---

### Step 4.3: Event Loop

Update `src/main.rs`:

```rust
mod app;
mod config;
mod error;
mod serial;
mod ui;

use app::{AppState, SharedState};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    // Create shared state
    let state: SharedState = Arc::new(Mutex::new(AppState::new()));

    // Create broadcast channel for serial messages
    let (serial_tx, _) = broadcast::channel::<serial::SerialMessage>(1000);

    // Spawn task to update display buffer from serial messages
    spawn_display_updater(state.clone(), serial_tx.subscribe());

    // Initialize terminal and run UI
    let terminal = ratatui::init();
    let result = run_tui(terminal, state).await;
    ratatui::restore();  // Always restore terminal, even on error

    result
}

fn spawn_display_updater(
    state: SharedState,
    mut rx: broadcast::Receiver<serial::SerialMessage>,
) {
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let mut s = state.lock().await;
            s.display_buffer.push_back(msg);

            // Cap buffer size to prevent unbounded memory growth
            while s.display_buffer.len() > 10000 {
                s.display_buffer.pop_front();
            }
        }
    });
}

async fn run_tui(mut terminal: DefaultTerminal, state: SharedState) -> Result<()> {
    loop {
        // Render current state
        {
            let s = state.lock().await;
            terminal.draw(|frame| ui::app_ui::render(frame, &s))?;
            if !s.running {
                break;
            }
        }
        // Lock released here (guard dropped)

        // Poll for input with timeout
        // 16ms ≈ 60 FPS - fast enough for smooth UI, slow enough to not waste CPU
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press (not release)
                if key.kind == KeyEventKind::Press {
                    let mut s = state.lock().await;
                    match key.code {
                        KeyCode::Char('q') => s.running = false,
                        KeyCode::Char(c) => s.input_text.push(c),
                        KeyCode::Backspace => { s.input_text.pop(); }
                        KeyCode::Enter => s.input_text.clear(),
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}
```

#### Why Separate the Lock Scope?

```rust
{
    let s = state.lock().await;
    terminal.draw(...)?;
    if !s.running { break; }
}  // Lock released here

// Now we can do other things without holding the lock
if event::poll(...)? { ... }
```

Holding a lock during I/O (event polling) is bad:

- Other tasks can't access state while we wait for input
- Can cause deadlocks if another task needs the lock

By scoping the lock, we hold it only during rendering.

#### Why 16ms Poll Timeout?

```rust
event::poll(Duration::from_millis(16))?
```

- **16ms ≈ 60 FPS**: Smooth enough for UI updates
- **If input arrives**: Returns immediately, handles input
- **If no input**: Returns after 16ms, loops back to render

This balances responsiveness with CPU usage. Without timeout, you'd either:

- Block forever waiting for input (UI doesn't update)
- Spin in a tight loop (100% CPU)

#### Why Check `KeyEventKind::Press`?

```rust
if key.kind == KeyEventKind::Press {
```

Terminals can report:

- `Press`: Key pressed down
- `Release`: Key released
- `Repeat`: Key held down (auto-repeat)

Without this check, you'd handle the same keypress multiple times.

#### Why `ratatui::restore()` is Critical?

```rust
let terminal = ratatui::init();
let result = run_tui(terminal, state).await;
ratatui::restore();  // MUST call this
```

`ratatui::init()` puts the terminal in "raw mode":

- No line buffering
- No echo
- No Ctrl+C handling
- Alternate screen (hides previous content)

If you don't call `restore()`:

- Terminal stays in raw mode
- User can't type normally
- Previous screen content is lost

Even on panic, you want to restore. Consider using a guard:

```rust
struct TerminalGuard;
impl Drop for TerminalGuard {
    fn drop(&mut self) { ratatui::restore(); }
}
```

---

### Step 4.4: UI mod.rs

Create `src/ui/mod.rs`:

```rust
pub mod app_ui;
```

**Checkpoint:** `cargo run` should show a TUI. Press 'q' to quit. Type characters to see input.

---

## Phase 5: Channel Wiring

**Goal:** Connect all components with proper message passing.

### The Channel Architecture

```
┌─────────────┐  AppCommand   ┌──────────────┐  SerialCommand  ┌─────────────┐
│   UI Task   │──────────────▶│   Command    │────────────────▶│   Serial    │
│             │               │  Dispatcher  │                 │   Handler   │
└─────────────┘               └──────────────┘                 └──────┬──────┘
                                     │                                │
                              ┌──────┴──────┐                         │
                              │             │                   broadcast
                              ▼             ▼                         │
                         Notification   Script                        │
                           System       Engine                        │
                                          │                           │
                                          │ SerialCommand             │
                                          └───────────────────────────┤
                                                                      │
                    ┌─────────────────────────────────────────────────┘
                    │
                    ▼
            ┌───────────────┐
            │   broadcast   │──────┬──────────┬──────────────┐
            │   channel     │      │          │              │
            └───────────────┘      ▼          ▼              ▼
                              Display     Logger        Script
                              Updater                   (waitstr)
```

### Why This Topology?

1. **UI → Command Dispatcher**: UI doesn't directly control serial ports. It sends high-level commands ("send this text", "run this script") to a dispatcher.

2. **Command Dispatcher → Serial Handler**: Dispatcher translates AppCommands to SerialCommands. This separation means the UI doesn't know serial implementation details.

3. **Serial Handler → broadcast**: All serial data goes through one broadcast channel. Multiple consumers subscribe without coordination.

4. **Script Engine → Serial Handler**: Scripts send commands through the same SerialCommand channel as the dispatcher. Scripts are just another command source.

### Channel Types and Why

```rust
// mpsc: Multiple producers, single consumer
// Used when many sources send to one handler
let (app_cmd_tx, app_cmd_rx) = tokio::sync::mpsc::channel::<AppCommand>(100);

// broadcast: Multiple producers, multiple consumers
// Used when one source sends to many handlers
let (serial_tx, _) = tokio::sync::broadcast::channel::<SerialMessage>(1000);

// oneshot: Single message, then channel closes
// Used for one-time signals like "abort script"
let (abort_tx, abort_rx) = tokio::sync::oneshot::channel::<()>();
```

### AppCommand Enum

```rust
use std::path::PathBuf;

/// Commands from UI to the command dispatcher.
pub enum AppCommand {
    /// Shut down the application
    Quit,

    /// Send text to specified ports
    SendText {
        ports: Vec<String>,
        text: String
    },

    /// Start executing a script
    RunScript {
        path: PathBuf
    },

    /// Abort currently running script
    StopScript,

    /// Toggle debug screen visibility
    ToggleDebug,

    /// Clear the display buffer
    ClearDisplay,

    /// Save current configuration to file
    SaveConfig,
}
```

#### Why Separate AppCommand from SerialCommand?

**AppCommand** is high-level, user-facing:

- "Send this text to selected ports"
- "Run this script"
- "Quit the app"

**SerialCommand** is low-level, implementation:

- "Send these bytes to these ports"
- "Connect this port"
- "Disconnect this port"

The command dispatcher translates between them:

```rust
AppCommand::SendText { ports, text } => {
    let bytes = (text + &line_ending).into_bytes();
    serial_tx.send(SerialCommand::Send { port_names: ports, data: bytes }).await;
}
```

This separation means:

- UI doesn't know about line endings
- UI doesn't know about byte encoding
- Serial handler doesn't know about "scripts" or "quit"

### Command Dispatcher Implementation

```rust
use tokio::sync::mpsc;

pub async fn run_command_dispatcher(
    mut rx: mpsc::Receiver<AppCommand>,
    serial_tx: mpsc::Sender<SerialCommand>,
    notif_tx: mpsc::Sender<Notification>,
    state: SharedState,
) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            AppCommand::Quit => {
                state.lock().await.running = false;
                break;
            }

            AppCommand::SendText { ports, text } => {
                // Get line ending from config (or default)
                let line_ending = "\n";  // TODO: get from config
                let data = (text + line_ending).into_bytes();

                if let Err(e) = serial_tx.send(SerialCommand::Send {
                    port_names: ports,
                    data
                }).await {
                    let _ = notif_tx.send(Notification::error(
                        format!("Failed to send: {}", e)
                    )).await;
                }
            }

            AppCommand::ClearDisplay => {
                state.lock().await.display_buffer.clear();
                let _ = notif_tx.send(Notification::info("Display cleared")).await;
            }

            AppCommand::ToggleDebug => {
                let mut s = state.lock().await;
                s.debug_mode = !s.debug_mode;
            }

            // ... other commands
        }
    }
}
```

#### Why Not Just Handle Commands in the UI Task?

**Problems with handling commands in UI:**

```rust
// In UI task
match key.code {
    KeyCode::Enter => {
        // Bad: UI task waits for serial send
        serial_tx.send(cmd).await?;
    }
}
```

- UI blocks while sending
- UI knows about SerialCommand details
- Complex logic clutters UI code

**Benefits of dispatcher:**

- UI just sends AppCommand and continues
- Dispatcher handles translation, error handling, notifications
- Easier to test dispatcher in isolation

---

## Phase 6: Multi-Port Support

**Goal:** Handle multiple serial ports simultaneously.

### Port Manager Design

```rust
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

/// Manages multiple serial port tasks.
pub struct PortManager {
    /// Running port tasks keyed by port name
    ports: HashMap<String, PortHandle>,

    /// Broadcast sender for serial messages
    broadcast_tx: broadcast::Sender<SerialMessage>,

    /// Per-port write channels
    write_channels: HashMap<String, mpsc::Sender<Vec<u8>>>,
}

struct PortHandle {
    task: JoinHandle<()>,
    write_tx: mpsc::Sender<Vec<u8>>,
}
```

#### Why Per-Port Write Channels?

Reading is broadcast (one to many): one port sends to many consumers.
Writing is targeted (many to one): many sources send to one port.

```rust
// Reading: broadcast
let (broadcast_tx, _) = broadcast::channel(1000);
// All ports send to broadcast_tx
// Multiple receivers (display, logger, script) subscribe

// Writing: mpsc per port
let (write_tx, write_rx) = mpsc::channel(100);
// PortManager holds write_tx
// Port task holds write_rx
```

When `SerialCommand::Send { port_names: ["GPS", "Motor"], data }` arrives:

```rust
for name in port_names {
    if let Some(handle) = self.ports.get(&name) {
        handle.write_tx.send(data.clone()).await?;
    }
}
```

### Port Task with Read/Write

```rust
pub async fn run_port_task(
    config: PortConfig,
    broadcast_tx: broadcast::Sender<SerialMessage>,
    mut write_rx: mpsc::Receiver<Vec<u8>>,
) {
    loop {
        match connect_and_run(&config, &broadcast_tx, &mut write_rx).await {
            Ok(()) => break,
            Err(e) => {
                eprintln!("[{}] Error: {}", config.name, e);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
}

async fn connect_and_run(
    config: &PortConfig,
    broadcast_tx: &broadcast::Sender<SerialMessage>,
    write_rx: &mut mpsc::Receiver<Vec<u8>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let port = tokio_serial::new(&config.path, config.baud_rate)
        .open_native_async()?;

    // Split into reader and writer
    let (reader, mut writer) = tokio::io::split(port);
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        tokio::select! {
            // Read from port
            result = reader.read_line(&mut line) => {
                let bytes_read = result?;
                if bytes_read == 0 {
                    return Err("Disconnected".into());
                }

                let msg = SerialMessage {
                    timestamp: Utc::now(),
                    port_name: config.name.clone(),
                    port_path: config.path.clone(),
                    data: line.trim_end().to_string(),
                };
                let _ = broadcast_tx.send(msg);
                line.clear();
            }

            // Write to port
            Some(data) = write_rx.recv() => {
                writer.write_all(&data).await?;
            }
        }
    }
}
```

#### Why `tokio::select!`?

We need to simultaneously:

- Read from the serial port (blocking until data arrives)
- Write to the serial port (blocking until write channel has data)

`select!` waits for whichever happens first:

```rust
tokio::select! {
    result = reader.read_line(&mut line) => { /* handle read */ }
    Some(data) = write_rx.recv() => { /* handle write */ }
}
```

Without `select!`, you'd need separate read/write tasks and more channels.

#### Why `tokio::io::split`?

```rust
let (reader, writer) = tokio::io::split(port);
```

Serial ports are single resources. `split` creates separate reader/writer handles that can be used in the same `select!`.

Without splitting, you'd need `Arc<Mutex<Port>>` and careful lock management.

---

## Phase 7: Vim Navigation

**Goal:** Navigate the display buffer with vim-style keys.

### Why Vim Keys?

1. **Efficiency**: Navigate without leaving home row
2. **Familiarity**: Many developers already know vim
3. **Terminal tradition**: Fits the TUI aesthetic
4. **No mouse needed**: Works over SSH

### VimMode State Machine

```rust
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum VimMode {
    #[default]
    Normal,   // Navigation mode (j/k/gg/G)
    Insert,   // Typing mode (text goes to input)
    Command,  // : command mode
    Search,   // / search mode
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Focus {
    #[default]
    Display,  // Focus on serial output (navigation)
    InputBox, // Focus on input box (typing)
}
```

#### Why Separate Mode and Focus?

**Mode** = how keys are interpreted
**Focus** = which widget receives input

Examples:

- Focus=Display, Mode=Normal: j/k scroll display
- Focus=Display, Mode=Search: typing goes to search box
- Focus=InputBox, Mode=Normal: h/l move cursor in input
- Focus=InputBox, Mode=Insert: typing goes to input

### Key Handling

```rust
pub struct KeyState {
    /// For multi-key sequences like 'gg'
    pending: Option<char>,

    /// Current mode
    mode: VimMode,

    /// Current focus
    focus: Focus,
}

impl KeyState {
    pub fn handle_key(&mut self, key: KeyCode, state: &mut AppState) -> Option<Action> {
        match (self.focus, self.mode) {
            (Focus::Display, VimMode::Normal) => self.handle_display_normal(key, state),
            (Focus::Display, VimMode::Search) => self.handle_display_search(key, state),
            (Focus::InputBox, VimMode::Normal) => self.handle_input_normal(key, state),
            (Focus::InputBox, VimMode::Insert) => self.handle_input_insert(key, state),
            // ...
        }
    }

    fn handle_display_normal(&mut self, key: KeyCode, state: &mut AppState) -> Option<Action> {
        match (self.pending, key) {
            // Multi-key: gg = go to top
            (Some('g'), KeyCode::Char('g')) => {
                self.pending = None;
                let max = state.display_buffer.len().saturating_sub(1);
                state.scroll_offset = max;
                None
            }

            // First g: wait for second key
            (None, KeyCode::Char('g')) => {
                self.pending = Some('g');
                None
            }

            // G = go to bottom
            (_, KeyCode::Char('G')) => {
                self.pending = None;
                state.scroll_offset = 0;
                None
            }

            // j = scroll down (show newer)
            (_, KeyCode::Char('j')) => {
                self.pending = None;
                state.scroll_offset = state.scroll_offset.saturating_sub(1);
                None
            }

            // k = scroll up (show older)
            (_, KeyCode::Char('k')) => {
                self.pending = None;
                state.scroll_offset += 1;
                None
            }

            // Ctrl+d = half page down
            (_, KeyCode::Char('d')) if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.pending = None;
                let half_page = 10; // TODO: calculate from viewport
                state.scroll_offset = state.scroll_offset.saturating_sub(half_page);
                None
            }

            // / = start search
            (_, KeyCode::Char('/')) => {
                self.pending = None;
                self.mode = VimMode::Search;
                state.search_query.clear();
                None
            }

            // i = switch to input box insert mode
            (_, KeyCode::Char('i')) => {
                self.pending = None;
                self.focus = Focus::InputBox;
                self.mode = VimMode::Insert;
                None
            }

            // : = command mode
            (_, KeyCode::Char(':')) => {
                self.pending = None;
                self.mode = VimMode::Command;
                state.command_input.clear();
                None
            }

            // Unknown key: clear pending
            _ => {
                self.pending = None;
                None
            }
        }
    }
}
```

#### Why a Pending Key State?

Vim has multi-key commands:

- `gg` = go to top
- `dd` = delete line
- `yy` = yank line

When user presses `g`, we don't know yet if they want `gg` (top) or `G` (bottom). We store the `g` as pending and wait for the next key.

Timeout handling (optional): If no second key within 500ms, treat pending key as standalone.

---

## Phase 8: Notifications

**Goal:** Display transient messages to the user.

### Why Notifications?

Users need feedback for:

- "Connected to GPS"
- "Config saved"
- "Script finished"
- "Error: Port not found"

Notifications appear briefly, then disappear. They don't interrupt workflow.

### Duration Calculation

```rust
impl Notification {
    pub fn new(message: impl Into<String>, level: NotificationLevel) -> Self {
        let msg = message.into();

        // Reading speed: ~15 characters per second
        // Add 1 second buffer for reaction time
        let duration = (msg.len() as f32 / 15.0) + 1.0;

        Self {
            message: msg,
            level,
            created_at: Instant::now(),
            duration_secs: duration,
        }
    }
}
```

#### Why Calculate Duration from Length?

Fixed duration is bad:

- "OK" visible for 5 seconds = wasteful
- "Error: Connection refused to /dev/ttyUSB0 (permission denied)" visible for 2 seconds = can't read it

Dynamic duration:

- Short messages = short display
- Long messages = long display
- Users can always read the full message

### Stacking Notifications

```rust
fn render_notifications(frame: &mut Frame, notifications: &VecDeque<Notification>) {
    let area = frame.area();
    let width = 40;

    let mut y = 1;
    for notif in notifications.iter().filter(|n| !n.is_expired()) {
        let x = area.width.saturating_sub(width + 1);
        let rect = Rect::new(x, y, width, 1);

        let style = match notif.level {
            NotificationLevel::Info => Style::default().fg(Color::Green),
            NotificationLevel::Warning => Style::default().fg(Color::Yellow),
            NotificationLevel::Error => Style::default().fg(Color::Red),
        };

        frame.render_widget(Paragraph::new(&*notif.message).style(style), rect);
        y += 1;

        if y > 5 { break; } // Max 5 visible notifications
    }
}
```

Multiple notifications stack vertically from the top-right. Oldest at top, newest at bottom.

---

## Phase 9: Logger Module

**Goal:** Write serial data to log files.

### Why Two Log Types?

**Combined log (`all.log`):**

- All ports interleaved
- See the full conversation
- Good for debugging interactions

**Per-port logs (`GPS.log`, `Motor.log`):**

- Only one port's data
- Easier to grep/analyze
- Smaller files

### Async File Writing

```rust
pub async fn run_logger(
    mut rx: broadcast::Receiver<SerialMessage>,
    log_dir: PathBuf,
) {
    let _ = tokio::fs::create_dir_all(&log_dir).await;

    let all_log = open_log_file(&log_dir.join("all.log")).await;
    let mut port_logs: HashMap<String, File> = HashMap::new();

    while let Ok(msg) = rx.recv().await {
        let line = format!(
            "<{}>[{}] {}\n",
            msg.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
            msg.port_name,
            msg.data
        );

        // Write to combined log
        write_line(&all_log, &line).await;

        // Write to per-port log (create if needed)
        let port_log = port_logs
            .entry(msg.port_name.clone())
            .or_insert_with(|| {
                let path = log_dir.join(format!("{}.log", msg.port_name));
                // Note: This blocks. In production, use async initialization.
                futures::executor::block_on(open_log_file(&path))
            });
        write_line(port_log, &line).await;
    }
}
```

#### Why Async File I/O?

Blocking file I/O in async code is bad:

```rust
std::fs::write(path, data)?;  // Blocks the entire thread
```

Tokio's file I/O runs on a thread pool:

```rust
tokio::fs::write(path, data).await?;  // Yields while writing
```

The task yields during I/O, letting other tasks run.

---

## Phase 10: Script Engine

**Goal:** Execute `.stui` scripts for automation.

### Why Build a Scripting Language?

**Alternative 1: Lua/Python embedding**

- Pros: Mature, well-documented
- Cons: Large dependency, complex FFI, security concerns

**Alternative 2: TOML/JSON command lists**

- Pros: Simple parsing
- Cons: No variables, no loops, no conditionals

**Alternative 3: Custom language (chosen)**

- Pros: Tailored to serial automation, small, secure
- Cons: Must implement from scratch

### The Three Stages

```
Source Code → [Lexer] → Tokens → [Parser] → AST → [Interpreter] → Execution
```

#### Stage 1: Lexer (Tokenizer)

Converts text to tokens:

```
"let x = 1 + 2;"
    ↓
[Let, Ident("x"), Equals, Number(1), Plus, Number(2), Semicolon]
```

Why tokens? Parser doesn't care about whitespace, comments, or exact syntax. Tokens are structured data.

#### Stage 2: Parser

Converts tokens to AST (Abstract Syntax Tree):

```
Let {
    name: "x",
    value: BinaryOp {
        left: Number(1),
        op: Plus,
        right: Number(2)
    }
}
```

Why AST? Execution order is explicit. Operator precedence is resolved. Easy to interpret.

#### Stage 3: Interpreter

Walks the AST and executes:

```rust
fn eval_stmt(&mut self, stmt: Stmt) -> Result<()> {
    match stmt {
        Stmt::Let { name, value } => {
            let val = self.eval_expr(value)?;
            self.env.insert(name, val);
        }
        Stmt::If { cond, then_block, else_block } => {
            if self.eval_expr(cond)?.is_truthy() {
                self.eval_block(then_block)?;
            } else if let Some(block) = else_block {
                self.eval_block(block)?;
            }
        }
        // ...
    }
    Ok(())
}
```

### Built-in Functions

```rust
async fn call_builtin(&mut self, name: &str, args: Vec<Value>) -> Result<Value> {
    match name {
        "sendstr" => {
            let ports = args[0].as_string_array()?;
            let text = args[1].as_string()?;
            self.serial_tx.send(SerialCommand::Send {
                port_names: ports,
                data: text.into_bytes(),
            }).await?;
            Ok(Value::Null)
        }

        "wait" => {
            let secs = args[0].as_number()?;
            tokio::time::sleep(Duration::from_secs_f64(secs)).await;
            Ok(Value::Null)
        }

        "waitstr" => {
            let ports = args[0].as_string_array()?;
            let pattern = args[1].as_string()?;
            let timeout = args[2].as_number()?;

            let regex = Regex::new(&pattern)?;
            let mut rx = self.serial_broadcast.subscribe();

            let result = tokio::time::timeout(
                Duration::from_secs_f64(timeout),
                async {
                    while let Ok(msg) = rx.recv().await {
                        if ports.contains(&msg.port_name) && regex.is_match(&msg.data) {
                            return Some(msg.data);
                        }
                    }
                    None
                }
            ).await;

            match result {
                Ok(Some(data)) => Ok(Value::String(data)),
                Ok(None) => Err("No match found".into()),
                Err(_) => Err("Timeout waiting for pattern".into()),
            }
        }

        _ => Err(format!("Unknown function: {}", name).into()),
    }
}
```

#### Why `waitstr` is Powerful

```rust
// Wait for GPS to respond with OK or ACK within 5 seconds
waitstr(["GPS"], r"OK|ACK", 5.0);
```

This single function:

1. Subscribes to the serial broadcast
2. Filters for specified ports
3. Matches with regex
4. Times out if no match

Without it, you'd need complex polling logic.

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let tokens = lex("let x = 1;");
        assert_eq!(tokens, vec![
            Token::Let,
            Token::Ident("x".into()),
            Token::Equals,
            Token::Number(1.0),
            Token::Semicolon,
        ]);
    }

    #[test]
    fn test_parser() {
        let ast = parse("let x = 1 + 2;");
        // Assert AST structure...
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_serial_broadcast() {
    let (tx, mut rx1) = broadcast::channel(10);
    let mut rx2 = tx.subscribe();

    tx.send(SerialMessage { ... }).unwrap();

    assert!(rx1.recv().await.is_ok());
    assert!(rx2.recv().await.is_ok());
}
```

### Manual Testing

1. **Virtual serial ports**: Use `socat` to create loopback

   ```bash
   socat -d -d pty,raw,echo=0 pty,raw,echo=0
   ```

2. **Real hardware**: Connect actual devices

3. **Stress testing**: Send high-volume data, test reconnection

---

## Common Pitfalls

### 1. Mutex Deadlock

**Bad:**

```rust
let mut guard = state.lock().await;
some_async_function().await;  // Still holding lock!
guard.value = 1;
```

**Good:**

```rust
{
    let mut guard = state.lock().await;
    guard.value = 1;
}  // Lock released
some_async_function().await;
```

### 2. Broadcast Lag

Receivers can fall behind:

```rust
match rx.recv().await {
    Ok(msg) => { /* handle */ }
    Err(broadcast::error::RecvError::Lagged(n)) => {
        eprintln!("Missed {} messages", n);
    }
}
```

### 3. Forgetting Terminal Restore

Always restore, even on panic:

```rust
let _guard = scopeguard::guard((), |_| {
    ratatui::restore();
});
```

### 4. Blocking in Async

**Bad:**

```rust
std::thread::sleep(Duration::from_secs(1));  // Blocks thread
```

**Good:**

```rust
tokio::time::sleep(Duration::from_secs(1)).await;  // Yields
```

---

## Resources

- [Tokio Tutorial](https://tokio.rs/tokio/tutorial) - Async Rust fundamentals
- [Ratatui Book](https://ratatui.rs/) - TUI framework guide
- [Crafting Interpreters](https://craftinginterpreters.com/) - Build a scripting language
- [tokio-serial Docs](https://docs.rs/tokio-serial/) - Serial port API
- [The Rust Book](https://doc.rust-lang.org/book/) - Rust fundamentals

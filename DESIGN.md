# SerialTUI Implementation Guide

A step-by-step guide to building SerialTUI from scratch.

---

## Overview

SerialTUI is a terminal UI for serial port communication with:
- Multi-port support (up to 255 ports)
- Vim-style navigation
- Custom scripting language
- Async architecture with tokio

**Architecture Summary:**
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

---

## Phase 1: Project Foundation

**Goal:** Compile, run, handle errors properly.

### Step 1.1: Dependencies

Add to `Cargo.toml`:

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

### Step 1.2: Directory Structure

Create this structure:

```
src/
├── main.rs
├── app.rs              # AppState, main loop
├── error.rs            # Custom error types
│
├── config/
│   ├── mod.rs
│   ├── port_config.rs  # PortConfig struct
│   └── manager.rs      # Load/save config
│
├── serial/
│   ├── mod.rs
│   ├── message.rs      # SerialMessage struct
│   ├── command.rs      # SerialCommand enum
│   ├── handler.rs      # Port manager task
│   └── port.rs         # Single port task
│
├── ui/
│   ├── mod.rs
│   ├── app_ui.rs       # Main render function
│   ├── display.rs      # Serial output display
│   ├── input_box.rs    # Text input widget
│   └── vim/
│       ├── mod.rs
│       └── mode.rs     # VimMode state
│
├── logger/
│   ├── mod.rs
│   └── writer.rs       # Log file writer
│
└── notification/
    ├── mod.rs
    └── system.rs       # Notification queue
```

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

    // TODO: Initialize app
    println!("SerialTUI starting...");

    Ok(())
}
```

**Checkpoint:** `cargo build` should succeed.

---

## Phase 2: Config Module

**Goal:** Load port configuration from TOML file.

### Step 2.1: PortConfig Struct

Create `src/config/port_config.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConfig {
    pub name: String,
    pub path: String,

    #[serde(default = "default_baud")]
    pub baud_rate: u32,

    #[serde(default = "default_line_ending")]
    pub line_ending: String,

    #[serde(default = "random_color")]
    pub color: String,
}

fn default_baud() -> u32 { 115200 }
fn default_line_ending() -> String { "\n".to_string() }
fn random_color() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("#{:02X}{:02X}{:02X}",
        rng.gen_range(100..255),
        rng.gen_range(100..255),
        rng.gen_range(100..255))
}
```

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
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Config { port: vec![] });
        }
        let content = std::fs::read_to_string(path)
            .map_err(|e| AppError::Config(e.to_string()))?;
        toml::from_str(&content)
            .map_err(|e| AppError::Config(e.to_string()))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| AppError::Config(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
```

### Step 2.3: Config mod.rs

Create `src/config/mod.rs`:

```rust
mod manager;
mod port_config;

pub use manager::Config;
pub use port_config::PortConfig;
```

**Checkpoint:** Create `config/config.toml` with test data:

```toml
[[port]]
name = "Test"
path = "/dev/ttyUSB0"
```

Load and print it from main.

---

## Phase 3: Serial Module (Single Port)

**Goal:** Read from one serial port asynchronously.

### Step 3.1: SerialMessage

Create `src/serial/message.rs`:

```rust
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct SerialMessage {
    pub timestamp: DateTime<Utc>,
    pub port_name: String,
    pub port_path: String,
    pub data: String,
}
```

### Step 3.2: SerialCommand

Create `src/serial/command.rs`:

```rust
#[derive(Debug, Clone)]
pub enum SerialCommand {
    Send { port_names: Vec<String>, data: Vec<u8> },
    Connect { port_name: String },
    Disconnect { port_name: String },
}
```

### Step 3.3: Single Port Task

Create `src/serial/port.rs`:

```rust
use super::message::SerialMessage;
use crate::config::PortConfig;
use chrono::Utc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::broadcast;
use tokio_serial::SerialPortBuilderExt;

pub async fn run_port_task(
    config: PortConfig,
    tx: broadcast::Sender<SerialMessage>,
) {
    loop {
        match connect_and_read(&config, &tx).await {
            Ok(()) => break, // Clean shutdown
            Err(e) => {
                eprintln!("Port {} error: {}", config.name, e);
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                // Reconnect
            }
        }
    }
}

async fn connect_and_read(
    config: &PortConfig,
    tx: &broadcast::Sender<SerialMessage>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = tokio_serial::new(&config.path, config.baud_rate)
        .open_native_async()?;

    let mut reader = BufReader::new(port);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;

        if bytes_read == 0 {
            return Err("Port disconnected".into());
        }

        let msg = SerialMessage {
            timestamp: Utc::now(),
            port_name: config.name.clone(),
            port_path: config.path.clone(),
            data: line.trim_end().to_string(),
        };

        let _ = tx.send(msg); // Ignore if no receivers
    }
}
```

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

**Checkpoint:** Spawn a port task, subscribe to broadcast, print messages.

---

## Phase 4: Basic TUI

**Goal:** Display serial messages in a terminal UI.

### Step 4.1: AppState

Create `src/app.rs`:

```rust
use crate::config::PortConfig;
use crate::serial::SerialMessage;
use std::collections::{HashMap, VecDeque};
use tokio::sync::Mutex;
use std::sync::Arc;

pub struct AppState {
    pub port_configs: HashMap<String, PortConfig>,
    pub display_buffer: VecDeque<SerialMessage>,
    pub scroll_offset: usize,
    pub input_text: String,
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

pub type SharedState = Arc<Mutex<AppState>>;
```

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

pub fn render(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // Display
            Constraint::Length(3), // Input
        ])
        .split(frame.area());

    // Display area
    render_display(frame, state, chunks[0]);

    // Input area
    render_input(frame, state, chunks[1]);
}

fn render_display(frame: &mut Frame, state: &AppState, area: ratatui::layout::Rect) {
    let height = area.height as usize - 2; // Account for borders
    let total = state.display_buffer.len();
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
        .block(Block::default().borders(Borders::ALL).title("Serial Output"));

    frame.render_widget(display, area);
}

fn render_input(frame: &mut Frame, state: &AppState, area: ratatui::layout::Rect) {
    let input = Paragraph::new(state.input_text.as_str())
        .block(Block::default().borders(Borders::ALL).title("Input"));
    frame.render_widget(input, area);
}
```

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

    // Setup state
    let state: SharedState = Arc::new(Mutex::new(AppState::new()));

    // Setup channels
    let (serial_tx, _) = broadcast::channel::<serial::SerialMessage>(1000);

    // Spawn display buffer updater
    let state_clone = state.clone();
    let mut serial_rx = serial_tx.subscribe();
    tokio::spawn(async move {
        while let Ok(msg) = serial_rx.recv().await {
            let mut s = state_clone.lock().await;
            s.display_buffer.push_back(msg);
            // Cap buffer size
            while s.display_buffer.len() > 10000 {
                s.display_buffer.pop_front();
            }
        }
    });

    // Run TUI
    let terminal = ratatui::init();
    let result = run_tui(terminal, state).await;
    ratatui::restore();

    result
}

async fn run_tui(mut terminal: DefaultTerminal, state: SharedState) -> Result<()> {
    loop {
        // Render
        {
            let s = state.lock().await;
            terminal.draw(|frame| ui::app_ui::render(frame, &s))?;
            if !s.running {
                break;
            }
        }

        // Handle input (non-blocking)
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
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

### Step 4.4: UI mod.rs

Create `src/ui/mod.rs`:

```rust
pub mod app_ui;
```

**Checkpoint:** `cargo run` shows a TUI with display and input areas.

---

## Phase 5: Channel Wiring

**Goal:** Connect all components with message passing.

### Key Channels

```rust
// Commands from UI to dispatcher
let (app_cmd_tx, app_cmd_rx) = tokio::sync::mpsc::channel::<AppCommand>(100);

// Commands to serial handler
let (serial_cmd_tx, serial_cmd_rx) = tokio::sync::mpsc::channel::<SerialCommand>(100);

// Serial data to all consumers
let (serial_broadcast_tx, _) = tokio::sync::broadcast::channel::<SerialMessage>(1000);

// Notifications from anywhere
let (notif_tx, notif_rx) = tokio::sync::mpsc::channel::<Notification>(100);
```

### AppCommand Enum

```rust
pub enum AppCommand {
    Quit,
    SendText { ports: Vec<String>, text: String },
    RunScript { path: PathBuf },
    StopScript,
    ToggleDebug,
    ClearDisplay,
}
```

### Command Dispatcher Pattern

```rust
async fn command_dispatcher(
    mut rx: mpsc::Receiver<AppCommand>,
    serial_tx: mpsc::Sender<SerialCommand>,
    state: SharedState,
) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            AppCommand::Quit => {
                state.lock().await.running = false;
                break;
            }
            AppCommand::SendText { ports, text } => {
                let _ = serial_tx.send(SerialCommand::Send {
                    port_names: ports,
                    data: text.into_bytes(),
                }).await;
            }
            // ... other commands
        }
    }
}
```

---

## Phase 6: Multi-Port Support

**Goal:** Handle multiple serial ports with a port manager.

### Port Manager

Create `src/serial/handler.rs`:

```rust
use super::{SerialCommand, SerialMessage};
use crate::config::PortConfig;
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

pub struct PortManager {
    ports: HashMap<String, JoinHandle<()>>,
    broadcast_tx: broadcast::Sender<SerialMessage>,
}

impl PortManager {
    pub fn new(broadcast_tx: broadcast::Sender<SerialMessage>) -> Self {
        Self {
            ports: HashMap::new(),
            broadcast_tx,
        }
    }

    pub fn connect(&mut self, config: PortConfig) {
        let tx = self.broadcast_tx.clone();
        let name = config.name.clone();

        let handle = tokio::spawn(async move {
            super::port::run_port_task(config, tx).await;
        });

        self.ports.insert(name, handle);
    }

    pub fn disconnect(&mut self, name: &str) {
        if let Some(handle) = self.ports.remove(name) {
            handle.abort();
        }
    }
}

pub async fn run_serial_handler(
    mut cmd_rx: mpsc::Receiver<SerialCommand>,
    broadcast_tx: broadcast::Sender<SerialMessage>,
    initial_configs: Vec<PortConfig>,
) {
    let mut manager = PortManager::new(broadcast_tx);

    // Connect initial ports
    for config in initial_configs {
        manager.connect(config);
    }

    // Handle commands
    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            SerialCommand::Connect { port_name } => {
                // Need to get config from somewhere
            }
            SerialCommand::Disconnect { port_name } => {
                manager.disconnect(&port_name);
            }
            SerialCommand::Send { port_names, data } => {
                // Route to specific ports (needs write channel per port)
            }
        }
    }
}
```

---

## Phase 7: Vim Navigation

**Goal:** Navigate display with vim keys.

### VimMode State

Create `src/ui/vim/mode.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VimMode {
    Normal,
    Insert,
    Command,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    Display,
    InputBox,
}
```

### Navigation Keys

| Key | Action | Implementation |
|-----|--------|----------------|
| `j` | Scroll down | `scroll_offset = scroll_offset.saturating_sub(1)` |
| `k` | Scroll up | `scroll_offset += 1` |
| `gg` | Top | `scroll_offset = max_offset` |
| `G` | Bottom | `scroll_offset = 0` |
| `Ctrl+d` | Half page down | `scroll_offset = scroll_offset.saturating_sub(height/2)` |
| `Ctrl+u` | Half page up | `scroll_offset += height/2` |

### Key Sequence Handling

```rust
struct KeyState {
    pending: Option<char>,  // For multi-char commands like 'gg'
}

fn handle_key(state: &mut AppState, key: KeyCode, keys: &mut KeyState) {
    match (keys.pending, key) {
        (Some('g'), KeyCode::Char('g')) => {
            // Jump to top
            keys.pending = None;
        }
        (None, KeyCode::Char('g')) => {
            keys.pending = Some('g');
        }
        (_, KeyCode::Char('G')) => {
            // Jump to bottom
            keys.pending = None;
        }
        _ => keys.pending = None,
    }
}
```

---

## Phase 8: Notifications

**Goal:** Display transient notifications.

### Notification Struct

```rust
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
    pub created_at: Instant,
    pub duration_secs: f32,
}

impl Notification {
    pub fn info(message: impl Into<String>) -> Self {
        let msg = message.into();
        let duration = (msg.len() as f32 / 15.0) + 1.0;
        Self {
            message: msg,
            level: NotificationLevel::Info,
            created_at: Instant::now(),
            duration_secs: duration,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_secs_f32() > self.duration_secs
    }
}
```

### Render at Top-Right

```rust
fn render_notifications(frame: &mut Frame, notifications: &VecDeque<Notification>) {
    let area = frame.area();
    let width = 40;

    for (i, notif) in notifications.iter().enumerate() {
        if notif.is_expired() { continue; }

        let y = 1 + i as u16;
        let x = area.width.saturating_sub(width + 1);

        let rect = Rect::new(x, y, width, 1);
        let style = match notif.level {
            NotificationLevel::Info => Style::default().fg(Color::Green),
            NotificationLevel::Warning => Style::default().fg(Color::Yellow),
            NotificationLevel::Error => Style::default().fg(Color::Red),
        };

        let para = Paragraph::new(&*notif.message).style(style);
        frame.render_widget(para, rect);
    }
}
```

---

## Phase 9: Logger Module

**Goal:** Write logs to files.

### Log Writer

```rust
use crate::serial::SerialMessage;
use chrono::Local;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::broadcast;

pub async fn run_logger(
    mut rx: broadcast::Receiver<SerialMessage>,
    log_dir: PathBuf,
) {
    // Ensure log directory exists
    let _ = tokio::fs::create_dir_all(&log_dir).await;

    // Open combined log
    let all_log_path = log_dir.join("all.log");
    let mut all_log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&all_log_path)
        .await
        .expect("Failed to open all.log");

    // Per-port logs
    let mut port_logs: HashMap<String, File> = HashMap::new();

    while let Ok(msg) = rx.recv().await {
        let line = format!(
            "<{}>[{}] {}\n",
            msg.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
            msg.port_name,
            msg.data
        );

        // Write to combined log
        let _ = all_log.write_all(line.as_bytes()).await;

        // Write to per-port log
        let port_log = port_logs.entry(msg.port_name.clone())
            .or_insert_with(|| {
                let path = log_dir.join(format!("{}.log", msg.port_name));
                futures::executor::block_on(async {
                    OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&path)
                        .await
                        .expect("Failed to open port log")
                })
            });
        let _ = port_log.write_all(line.as_bytes()).await;
    }
}
```

---

## Phase 10: Script Engine

**Goal:** Execute .stui scripts.

This is the most complex phase. Follow the structure:

### 10.1 Lexer (Tokenizer)

Converts source text into tokens:
```
"let x = 1 + 2;"  →  [Let, Ident("x"), Eq, Number(1), Plus, Number(2), Semi]
```

### 10.2 Parser (AST Generator)

Converts tokens into tree:
```
Let { name: "x", value: BinaryOp { left: 1, op: Plus, right: 2 } }
```

### 10.3 Interpreter (Executor)

Walks the tree and executes:
```
env["x"] = 3
```

### Built-in Functions

| Function | Behavior |
|----------|----------|
| `sendstr(ports, text)` | Send `SerialCommand::Send` |
| `sendbin(ports, hex)` | Parse hex, send as bytes |
| `wait(secs)` | `tokio::time::sleep` |
| `waitstr(ports, regex, timeout)` | Subscribe to broadcast, match with timeout |

### Script Execution Flow

```
:run script.stui
       │
       ▼
   Load file
       │
       ▼
   Lexer → Tokens
       │
       ▼
   Parser → AST
       │
       ▼
   Interpreter
       │
       ├──▶ sendstr() ──▶ serial_cmd_tx
       ├──▶ wait() ──▶ tokio::time::sleep
       └──▶ waitstr() ──▶ subscribe broadcast, match regex
```

---

## Implementation Order (Recommended)

### Week 1-2: Foundation
1. [ ] Set up Cargo.toml with all dependencies
2. [ ] Create directory structure and mod.rs files
3. [ ] Implement error types
4. [ ] Implement Config module (load/save TOML)
5. [ ] Verify: Load config, print port names

### Week 3-4: Serial + Basic TUI
6. [ ] Implement SerialMessage and SerialCommand
7. [ ] Implement single port read task
8. [ ] Set up broadcast channel
9. [ ] Implement basic TUI (display + input)
10. [ ] Verify: See serial data in TUI

### Week 5-6: Multi-Port + Notifications
11. [ ] Implement PortManager (multi-port)
12. [ ] Implement port selector UI
13. [ ] Implement notification system
14. [ ] Add connection/disconnection notifications
15. [ ] Verify: Multiple ports, notifications appear

### Week 7-8: Vim Navigation
16. [ ] Implement VimMode state machine
17. [ ] Implement j/k/gg/G navigation
18. [ ] Implement Ctrl+d/Ctrl+u
19. [ ] Implement search (/, n, N)
20. [ ] Implement yank (y, yy)
21. [ ] Verify: Navigate and search display

### Week 9-10: Config UI + Commands
22. [ ] Implement config bar (top)
23. [ ] Implement popups (add port, color picker)
24. [ ] Implement : commands (:q, :w, :debug, etc.)
25. [ ] Verify: Add/edit ports via UI

### Week 11-14: Script Engine
26. [ ] Implement Lexer
27. [ ] Implement Parser
28. [ ] Implement Interpreter (basic: let, if, while)
29. [ ] Implement built-ins (sendstr, wait)
30. [ ] Implement waitstr with broadcast subscription
31. [ ] Verify: Run test script

### Week 15+: Polish
32. [ ] Auto-reconnect with backoff
33. [ ] Debug screen
34. [ ] :memory and :thread commands
35. [ ] Macro system
36. [ ] Edge cases and performance

---

## Testing Strategy

### Unit Tests
- Config parsing
- Lexer token generation
- Parser AST generation
- Notification timing

### Integration Tests
- Serial read/write with loopback
- Channel communication
- Script execution

### Manual Testing
- Connect real serial devices
- Test reconnection (unplug/replug)
- Test script abort

---

## Key Patterns to Remember

### 1. Shared State Pattern
```rust
let state: Arc<Mutex<AppState>> = Arc::new(Mutex::new(AppState::new()));
```

### 2. Task Spawning Pattern
```rust
tokio::spawn(async move {
    // Task code
});
```

### 3. Channel Select Pattern
```rust
tokio::select! {
    Some(cmd) = cmd_rx.recv() => { /* handle command */ }
    Ok(msg) = serial_rx.recv() => { /* handle message */ }
}
```

### 4. Graceful Shutdown
```rust
let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
// In task: while !*shutdown_rx.borrow() { ... }
// To shutdown: shutdown_tx.send(true);
```

---

## Common Pitfalls

1. **Mutex deadlock**: Don't hold lock across await points
2. **Broadcast lag**: Receivers can fall behind; handle `RecvError::Lagged`
3. **Terminal restore**: Always restore terminal on panic (use `ratatui::restore`)
4. **Port permissions**: May need `sudo` or user group for serial ports
5. **Blocking in async**: Don't use `std::thread::sleep`, use `tokio::time::sleep`

---

## Resources

- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Ratatui Book](https://ratatui.rs/)
- [Crafting Interpreters](https://craftinginterpreters.com/) (for script engine)
- [tokio-serial Docs](https://docs.rs/tokio-serial/)

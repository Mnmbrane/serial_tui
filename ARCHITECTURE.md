# SerialTUI Architecture

## Overview

```
┌───────────┐     ┌────────────┐     ┌──────┐
│  UI Loop  │────>│  SerialHub │────>│ Port │ (per port)
│ (ratatui) │     │            │     └──────┘
└───────────┘     └────────────┘
      ^                 │                │
      │        unbounded mpsc            │
      └──────────────────────────────────┘

                  ┌─────────────┐
Port reader ─────>│ Logger Task │──> per-port .log files
  (log_tx)        │             │──> super.log
                  └─────────────┘
```

## Directory Structure

```
src/
├── main.rs              # Entry point, runtime setup
├── app.rs               # App struct, channel wiring, task spawning
├── error.rs             # ConfigError enum (thiserror)
├── notify.rs            # Notify struct, NotifyLevel enum
├── logger.rs            # Logger task (per-port + super.log)
│
├── config/
│   └── port.rs          # PortConfig, LineEnding enum
│
├── types/
│   └── color.rs         # Color wrapper with serde support
│
├── serial/
│   ├── hub.rs           # SerialHub: multi-port manager
│   ├── port.rs          # Port: reader/writer tokio tasks, PortEvent
│   └── error.rs         # SerialError enum
│
└── ui/
    ├── ui.rs            # Main render loop, event routing
    ├── widgets/
    │   ├── config_bar.rs   # Top bar with port controls
    │   ├── display.rs      # Serial output (vim nav, search, yank)
    │   └── input_bar.rs    # Text input bar
    └── popup/
        ├── notification.rs # Toast messages
        ├── port_list.rs    # Port selection popup
        └── send_group.rs   # Send target selection
```

## Data Flow

### Port Opening
```
App::new()
  └─> SerialHub::new(notify_tx, log_tx)
        └─> hub.load_config(ports.toml)
              └─> hub.open(name, PortConfig)
                    └─> Port::open(name, config, event_tx, log_tx, notify_tx)
                          ├─> spawn reader task (tokio::spawn)
                          └─> spawn writer task (tokio::spawn)
```

### Receiving Data
```
Serial Port
  └─> Reader task (async loop)
        └─> read bytes into buffer
              └─> Arc<PortEvent::Data { port, data, timestamp }>
                    ├─> event_tx.send() ──> UI display
                    └─> log_tx.send()   ──> Logger task
```

### Logging
```
Logger task
  └─> recv Arc<PortEvent::Data>
        ├─> logs/<port>.log  : [HH:MM:SS.mmm] <data>
        └─> logs/super.log   : [HH:MM:SS.mmm] [port] <data>
```

### Sending Data
```
User types + Enter
  └─> InputBarAction::Send(text)
        ├─> (empty)  hub.send_line_ending(ports)
        │               └─> per-port: send configured line ending
        └─> (text)   hub.send(ports, data)
                        └─> writer task ──> serial port
```

### Notifications
```
Background task error
  └─> notify_tx.send(Notify { level, source, message })
        └─> UI drains notify_rx ──> toast popup
```

## Key Components

### App
- Creates all channels (notify, log, serial events)
- Wires SerialHub, Logger, and UI together
- Spawns the logger as a tokio task

### SerialHub
- Owns all port connections (`HashMap<Arc<str>, Port>`)
- Unbounded mpsc channel for received data
- `send()` sends raw data to selected ports
- `send_line_ending()` sends each port's configured line ending

### Port
- `Port::open()` spawns reader + writer as tokio tasks
- Reader: async read loop, timestamps each read with `Local::now()`
- Writer: receives `Bytes` via mpsc, writes to serial port
- Errors sent to notify channel (not event channel)

### Logger
- Standalone tokio task receiving `Arc<PortEvent>` via unbounded mpsc
- Creates `logs/` directory on startup
- Opens per-port `.log` files lazily on first data
- Writes to both per-port file and `super.log`

### Display Widget
- VecDeque<Line> circular buffer (max 10,000 lines)
- Cursor-based scrolling with 25% margin auto-scroll
- Vim navigation: j/k, gg/G, Ctrl+u/d
- Search mode: /, n/N for matches
- Visual selection: v to toggle, y to yank
- Clipboard via arboard (kept alive for Linux)

### UI Event Loop
```rust
while !self.exit {
    terminal.draw(|f| self.draw(f))?;

    // Process serial events
    while let Ok(event) = self.serial_rx.try_recv() {
        // Add to display
    }

    // Drain notifications
    while let Ok(notif) = self.notify_rx.try_recv() {
        // Show toast
    }

    // Poll keyboard (16ms = ~60fps)
    if event::poll(Duration::from_millis(16))? {
        self.handle_key(key);
    }
}
```

## Channel Architecture

All channels are **unbounded mpsc** (`tokio::sync::mpsc::unbounded_channel`):

| Channel | Type | Producer | Consumer |
|---------|------|----------|----------|
| event | `Arc<PortEvent>` | Port reader tasks | UI |
| log | `Arc<PortEvent>` | Port reader tasks | Logger task |
| notify | `Notify` | Any background task | UI |
| writer (per-port) | `Bytes` | UI (via hub.send) | Port writer task |

The writer channel is the only **bounded** mpsc (capacity: 32) to provide backpressure on sends.

## Concurrency Model

```
Main Thread (UI)
  ├─> Render loop
  ├─> Keyboard handling
  ├─> Drains serial events (unbounded mpsc)
  └─> Drains notifications (unbounded mpsc)

Per-Port Reader Task (tokio::spawn)
  └─> Loop: async read -> Arc<PortEvent> -> event_tx + log_tx

Per-Port Writer Task (tokio::spawn)
  └─> Loop: recv from mpsc -> async write

Logger Task (tokio::spawn)
  └─> Loop: recv from log_rx -> write to log files
```

## Configuration

### config/ports.toml
```toml
[port_name]
path = "/dev/ttyUSB0"
baud_rate = 115200
line_ending = "lf"      # lf, cr, crlf
color = "green"         # or "#RRGGBB"
```

### PortConfig Defaults
- baud_rate: 115200
- line_ending: LF
- color: Reset

## Dependencies

| Crate | Purpose |
|-------|---------|
| ratatui | TUI framework |
| crossterm | Terminal backend |
| tokio-serial | Async serial I/O |
| tokio | Async runtime, channels, tasks |
| chrono | Timestamps |
| bytes | Efficient byte buffers |
| serde + toml | Config parsing |
| arboard | Clipboard |
| thiserror | Error types |
| anyhow | Error propagation |
| regex | Search patterns |

# SerialTUI Architecture

## Overview

```
┌─────────────┐     ┌─────────────────┐     ┌──────────────┐
│   UI Task   │────>│  SerialManager  │────>│ PortConnection│
│  (ratatui)  │     │   (pub/sub)     │     │ (per port)    │
└─────────────┘     └─────────────────┘     └──────────────┘
       ^                    │                      │
       │         broadcast channel                 │
       └───────────────────────────────────────────┘
```

## Directory Structure

```
src/
├── main.rs              # Entry point, config creation
├── error.rs             # AppError enum (thiserror)
│
├── types/
│   └── color.rs         # Color wrapper with serde support
│
├── serial/
│   ├── serial_manager.rs   # Multi-port manager, broadcast hub
│   ├── port_connection.rs  # Reader/writer threads per port
│   ├── port_handle.rs      # Low-level serialport wrapper
│   └── port_info.rs        # PortInfo config, LineEnding enum
│
└── ui/
    ├── ui.rs               # Main render loop, event routing
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
main.rs
  └─> SerialManager::from_file(config.toml)
        └─> SerialManager::open(name, PortInfo)
              ├─> PortHandle::open(path, baud_rate)
              ├─> spawn_reader(handle, broadcast_tx)
              └─> spawn_writer(handle, mpsc_rx)
```

### Receiving Data
```
Serial Port
  └─> Reader Thread (10ms timeout loop)
        └─> Buffer bytes until \n or \r
              └─> broadcast.send(PortEvent::Data)
                    └─> UI receives via serial_rx
                          └─> Display::push_line()
```

### Sending Data
```
User types + Enter
  └─> InputBarAction::Send(text)
        └─> SerialManager::send(ports, data)
              └─> Append line_ending per port
                    └─> mpsc.send() to writer thread
                          └─> PortHandle::write_all() + flush()
```

## Key Components

### SerialManager
- Owns all port connections
- Broadcast channel (capacity: 1024) for received data
- HashMap<String, ManagedPort> for port lookup
- `send()` appends configured line ending per port

### PortConnection
- Spawns reader thread (line-buffered, emits on \n/\r)
- Spawns writer thread (receives via mpsc, writes + flushes)
- Uses `Arc<str>` for port names (cheap cloning)
- Uses `bytes::Bytes` for data (cheap cloning)

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

    // Poll keyboard (16ms = ~60fps)
    if event::poll(Duration::from_millis(16))? {
        self.handle_key(key);
    }
}
```

## Threading Model

```
Main Thread (UI)
  ├─> Render loop
  ├─> Keyboard handling
  └─> Receives from broadcast

Per-Port Reader Thread
  └─> Loop: read -> buffer -> broadcast on newline

Per-Port Writer Thread
  └─> Loop: recv from mpsc -> write + flush
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

### PortInfo Defaults
- baud_rate: 115200
- line_ending: LF
- color: Reset

## Dependencies

| Crate | Purpose |
|-------|---------|
| ratatui | TUI framework |
| crossterm | Terminal backend |
| serialport | Serial I/O |
| tokio | Broadcast channel |
| serde + toml | Config |
| chrono | Timestamps |
| arboard | Clipboard |
| bytes | Efficient buffers |
| thiserror | Error types |

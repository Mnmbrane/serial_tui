# SerialTUI Todo

## Done

### Core
- [x] Project setup (Cargo.toml, directory structure)
- [x] Error handling with thiserror
- [x] Color type with serde support

### Config
- [x] PortInfo struct (path, baud_rate, line_ending, color)
- [x] LineEnding enum (LF, CR, CRLF)
- [x] TOML config loading
- [x] Default config creation on first run

### Serial
- [x] SerialManager (multi-port hub)
- [x] PortConnection (reader/writer threads)
- [x] PortHandle (serialport wrapper)
- [x] Broadcast channel for received data
- [x] Line buffering (emit on \n or \r)
- [x] Send with line ending append
- [x] Write flush for reliability

### UI Core
- [x] Ratatui render loop (~60fps)
- [x] Terminal setup/restore
- [x] Focus management (Tab to cycle)
- [x] Keyboard routing

### UI Widgets
- [x] ConfigBar (top, shows keybinds)
- [x] Display (middle, serial output)
- [x] InputBar (bottom, text input)

### Display Features
- [x] Circular buffer (10k lines max)
- [x] Timestamp + port name + colored output
- [x] Cursor-based scrolling (25% margin)
- [x] Vim navigation: j/k, gg/G, Ctrl+u/d
- [x] Search mode: /, n/N
- [x] Visual selection: v
- [x] Yank to clipboard: y

### Popups
- [x] Notification (toast, auto-dismiss)
- [x] PortListPopup (view ports)
- [x] SendGroupPopup (select send targets)

---

## In Progress

- [ ] Investigate socat PTY write reliability issue

---

## Backlog

### Serial
- [ ] Auto-reconnect with exponential backoff
- [ ] Port hot-plug detection
- [ ] Connection status indicator

### UI
- [ ] Add Port popup (create new port config)
- [ ] Edit Port popup
- [ ] Filter ports from display
- [ ] Preview panel (word-wrapped selected line)

### Input
- [ ] Command mode (:q, :w, :clear)
- [ ] Input history (up/down arrows)
- [ ] Vim-style cursor movement in input

### Logging
- [ ] Combined log file (all.log)
- [ ] Per-port log files

### Scripting (Future)
- [ ] Lexer
- [ ] Parser
- [ ] Interpreter
- [ ] Built-ins: sendstr, sendbin, wait, waitstr

### Config
- [ ] Macro keybindings (F1, F2, etc.)
- [ ] Save config changes

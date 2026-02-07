# SerialTUI Todo

## Done

### Core
- [x] Project setup (Cargo.toml, directory structure)
- [x] Error handling with thiserror
- [x] Color type with serde support
- [x] App struct with channel wiring

### Config
- [x] PortConfig struct (path, baud_rate, line_ending, color)
- [x] LineEnding enum (LF, CR, CRLF)
- [x] TOML config loading
- [x] Default config creation on first run

### Serial
- [x] SerialHub (multi-port manager)
- [x] Port (async reader/writer tokio tasks)
- [x] Unbounded mpsc channel for received data
- [x] Send raw data to selected ports
- [x] Send line ending on empty Enter (per-port config)
- [x] Write flush for reliability
- [x] Notify channel for background errors

### Logging
- [x] Logger tokio task (separate from notify channel)
- [x] Per-port log files (`logs/<port>.log`)
- [x] Combined super log (`logs/super.log`)
- [x] Timestamp captured at read time (shared with UI)

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

### Scripting (Future)
- [ ] Lexer
- [ ] Parser
- [ ] Interpreter
- [ ] Built-ins: sendstr, sendbin, wait, waitstr

### Config
- [ ] Macro keybindings (F1, F2, etc.)
- [ ] Save config changes

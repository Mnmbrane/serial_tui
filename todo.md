# SerialTUI Kanban

## Backlog

### Foundation

- [x] Project setup
  - [x] Cargo.toml
  - [x] directory structure
- [ ] Basic main.rs with tokio runtime

### Config Module

- [ ] Define `PortConfig` struct with serde
- [ ] Define `DefaultsConfig` struct (default port settings)
- [ ] Define `MacroConfig` struct (name, key, command)
- [ ] Config file parser (load config/config.toml)
- [ ] Config validation (unique names/paths, notify on error)
- [ ] Config save functionality
- [ ] Default value generation (random unique colors)
- [ ] Macro file parser (load config/macros.toml)

### Serial Module

- [ ] Define `SerialMessage` struct (timestamp, port_name, port_path, data)
- [ ] Define `SerialCommand` enum (Send, Connect, Disconnect)
- [ ] Port connection logic (tokio-serial)
- [ ] Line buffering with configurable delimiter
- [ ] Single port read task (async)
- [ ] Single port write handling
- [ ] Broadcast channel setup (fan-out to consumers)
- [ ] mpsc receiver for SerialCommand
- [ ] Auto-reconnect with exponential backoff (1s-8s cap)
- [ ] Port manager (spawn/despawn port tasks)
- [ ] Binary data handling for sendbin
- [ ] Connection restored notification

### Logger Module

- [ ] Log formatter (timestamp + port_name + data)
- [ ] Combined log writer (all.log)
- [ ] Per-port log writer (`{port}.log`)
- [ ] Async file writing (non-blocking)
- [ ] Log directory creation

### App Integration

- [ ] Define `AppState` struct
- [ ] Define `AppCommand` enum
- [ ] Channel wiring (app_command, serial_command, broadcast, notif)
- [ ] Command Dispatcher task (routes AppCommand)
- [ ] Graceful shutdown

### UI - Core

- [ ] Basic ratatui app loop
- [ ] Main layout (config bar / display+preview / input+selector)
- [ ] Focus management (Display vs InputBox)
- [ ] VimMode state machine (Normal, Insert, Command, Search)
- [ ] Keyboard event routing based on focus and mode

### UI - Config Bar (Top)

- [ ] Port dropdown widget
- [ ] Filter dropdown (hide ports from display)
- [ ] "Add Port" button
- [ ] "Run Script" dropdown
- [ ] Layout and styling

### UI - Display (Middle)

- [ ] Display buffer (`VecDeque<SerialMessage>`)
- [ ] Render interleaved lines with colors
- [ ] Timestamp formatting
- [ ] Port name coloring
- [ ] Line truncation (no wrap)
- [ ] Scroll position tracking
- [ ] Unlimited scroll history

### UI - Preview (Bottom of Display)

- [ ] Preview panel (3 lines default)
- [ ] Dynamic height expansion (upward)
- [ ] Word wrapping for selected line
- [ ] Show/hide on focus change

### UI - Vim Navigation

- [ ] Normal mode state
- [ ] `j`/`k` line scroll
- [ ] `Ctrl+d`/`Ctrl+u` half-page scroll
- [ ] `gg`/`G` top/bottom jump
- [ ] Line selection/cursor

### UI - Vim Search

- [ ] `/` search prompt
- [ ] Regex search engine (grep-regex)
- [ ] Highlight all matches
- [ ] `n`/`N` next/previous match
- [ ] Search state management

### UI - Vim Yank

- [ ] `y` yank visual selection
- [ ] `yy` yank current line
- [ ] Clipboard integration (arboard)
- [ ] Visual selection mode

### UI - Input Box (Bottom)

- [ ] Text input widget
- [ ] Insert mode (typing text)
- [ ] Normal mode (vim movement commands)
- [ ] Cursor movement (h/l/w/b/0/$)
- [ ] Basic editing (backspace, delete, x, dd)
- [ ] Command history
- [ ] `Ctrl+R` reverse search (display buffer)
- [ ] Enter sends text to selected ports
- [ ] `:` command parsing (:q, :w, :run, :stop, :debug, etc.)
- [ ] `:memory` and `:thread` send debug info to notification

### UI - Port Selector (Bottom Left)

- [ ] Checkbox list widget
- [ ] Scrollable container
- [ ] Toggle port selection
- [ ] Visual styling (checked/unchecked)

### UI - Popups

- [ ] Popup overlay system
- [ ] Add Port popup form
- [ ] Edit Port popup (same fields, for existing ports)
- [ ] Color picker popup
- [ ] Exit confirmation popup
- [ ] Popup keyboard handling

### UI - Notifications

- [ ] Notification struct (message, level, created_at)
- [ ] Duration calculation (char_count / reading_speed + 1s)
- [ ] Notification queue (stacking, VecDeque in AppState)
- [ ] Top-right rendering
- [ ] Auto-dismiss timer based on duration
- [ ] Notification levels (Info, Warning, Error)
- [ ] Notification System task (mpsc receiver)

### UI - Debug Screen

- [ ] Debug screen overlay (toggled via `:debug`)
- [ ] Debug log buffer (VecDeque in AppState)
- [ ] Capture debug prints to buffer
- [ ] Scroll within debug screen
- [ ] Vim navigation in debug screen

### Script Engine - Lexer

- [ ] Token definitions
- [ ] Number literals (f64)
- [ ] String literals (with escapes)
- [ ] Boolean literals
- [ ] Identifiers
- [ ] Keywords (let, if, elif, else, while, for, in, fn, return)
- [ ] Operators (+, -, *, /, %, ==, !=, <, >, <=, >=, &&, ||, !)
- [ ] Punctuation (; , ( ) [ ] { } ..)
- [ ] Raw string literals (r"...")
- [ ] Hex literals (0x...)
- [ ] Comments (// and /\*\*/)
- [ ] Error reporting with line/column

### Script Engine - Parser

- [ ] AST node definitions
- [ ] Expression parsing (precedence climbing)
- [ ] Binary operations
- [ ] Unary operations
- [ ] Function calls
- [ ] Array literals
- [ ] Array indexing
- [ ] Let statements
- [ ] Assignment statements
- [ ] If/elif/else statements
- [ ] While loops
- [ ] For loops (range-based)
- [ ] Function definitions
- [ ] Return statements
- [ ] Block parsing
- [ ] Error recovery and reporting

### Script Engine - Interpreter

- [ ] Environment/scope management
- [ ] Variable storage (f64, String, bool, Array)
- [ ] Expression evaluation
- [ ] Number operations (+, -, *, /, %)
- [ ] String operations (concatenation)
- [ ] Boolean operations (&&, ||, !)
- [ ] Comparison operations (==, !=, <, >, <=, >=)
- [ ] Array operations (indexing, iteration)
- [ ] Control flow (if/elif/else)
- [ ] Loop execution (while, for with range 0..n)
- [ ] Function calls (user-defined)
- [ ] Function return values
- [ ] Built-in: `sendstr(ports, string)`
- [ ] Built-in: `sendbin(ports, hex)`
- [ ] Built-in: `wait(seconds)`
- [ ] Built-in: `waitstr(ports, regex, timeout)`
- [ ] Script abort on waitstr timeout
- [ ] Script abort via oneshot channel
- [ ] Async execution (non-blocking wait/waitstr)
- [ ] Error handling (send notification on error)

### Macro System

- [ ] Macro execution from key binding (F1, F2, etc.)
- [ ] Parse macro command string
- [ ] Execute as script snippet

### Display Buffer Updater

- [ ] Async task subscribes to serial_broadcast
- [ ] Push SerialMessage to AppState.display_buffer
- [ ] Lock/unlock AppState mutex

### Polish

- [ ] Config error notifications (don't crash)
- [ ] Connection restored notifications
- [ ] Script running/done notifications
- [ ] Port disconnected notifications
- [ ] Keyboard shortcut help
- [ ] Edge case handling
- [ ] Performance optimization

---

## In Progress

(Move items here when you start working on them)

---

## Done

(Move items here when completed)

---

## Milestones

### M1: Foundation + Basic Serial

- Project setup (Cargo.toml, directory structure)
- Config module (load config.toml, PortConfig struct)
- Serial module (single port read task, SerialMessage)
- Logger module (combined all.log)
- Basic tokio main loop

### M2: Minimal TUI + Channel Wiring

- AppState struct
- Channel setup (broadcast, app_command, serial_command, notif)
- Basic ratatui loop
- Display buffer rendering (read from AppState)
- Display Buffer Updater task
- Input box (basic text input)
- Send text to single port

### M3: Multi-Port + Notifications

- Multiple port tasks
- Port selector checkboxes (bottom-left)
- Interleaved display with port name coloring
- Per-port logging
- Notification System task
- Basic notifications (connected, disconnected)

### M4: Vim Navigation + Preview

- Vim mode state machine (Normal, Insert)
- Display navigation (j/k/gg/G/Ctrl-d/Ctrl-u)
- Line selection/cursor
- Preview panel (3 lines, wrapping)
- Search (/, n, N, highlight all)
- Yank (y, yy to clipboard)

### M5: Config UI + Commands

- Config bar (top)
- Port dropdown
- Filter dropdown (hide ports)
- Add port popup
- Color picker popup
- Save config (:w)
- Commands (:q, :debug, :clear, :connect, :disconnect)

### M6: Script Engine

- Lexer (tokens, literals, operators)
- Parser (AST, expressions, statements, functions)
- Interpreter (variables, control flow, loops, functions)
- Built-ins (sendstr, sendbin, wait, waitstr)
- Script execution from UI (:run, :stop)
- Script notifications (started, finished, aborted)

### M7: Polish + Advanced Features

- Auto-reconnect with notification
- Debug screen (:debug toggle)
- :memory and :thread notifications
- Macro system (F1, F2, etc.)
- Ctrl+R reverse search in input box
- Config error handling (notify, don't crash)
- Edge case handling
- Performance optimization

---

## Notes

### Priority Order

1. M1: Foundation + Basic Serial (get data flowing to logs)
2. M2: Minimal TUI + Channel Wiring (see data, send data)
3. M3: Multi-Port + Notifications (scale to 255 ports)
4. M4: Vim Navigation + Preview (navigate and interact)
5. M5: Config UI + Commands (configure without editing files)
6. M6: Script Engine (automation)
7. M7: Polish + Advanced Features (production ready)

### Dependencies Diagram

```text
                ┌─────────────────────────────────────┐
                │         App Integration             │
                │  (AppState, Channels, Dispatcher)   │
                └─────────────────────────────────────┘
                                ▲
      ┌─────────────────────────┼─────────────────────────┐
      │                         │                         │
┌─────┴─────┐             ┌─────┴─────┐             ┌─────┴─────┐
│  Config   │             │  Serial   │             │  Logger   │
│  Module   │             │  Module   │             │  Module   │
└───────────┘             └───────────┘             └───────────┘
                                │
                broadcast::Sender<SerialMessage>
                                │
      ┌─────────────────────────┼─────────────────────────┐
      │                         │                         │
┌─────┴─────┐             ┌─────┴─────┐             ┌─────┴─────┐
│ Display   │             │  Logger   │             │  Script   │
│ Buffer    │             │ (writes)  │             │  Engine   │
│ Updater   │             │           │             │ (waitstr) │
└───────────┘             └───────────┘             └───────────┘
      │                                                   │
      └───────────────► UI Task ◄─────────────────────────┘
                     (reads AppState)
```

### Complexity Estimates

| Component              | Complexity | Notes                       |
|------------------------|------------|-----------------------------|
| Config                 | Low        | Serde does heavy lifting    |
| Serial                 | Medium     | Async + reconnect logic     |
| Logger                 | Low        | Straightforward file I/O    |
| App Integration        | Medium     | Channel wiring, Dispatcher  |
| UI Core                | Medium     | Layout + state management   |
| Vim Nav                | Medium     | Multiple modes + state      |
| Vim Search             | Medium     | Regex + highlighting        |
| Popups                 | Medium     | Overlay system              |
| Notifications          | Low        | Timer + queue               |
| Display Buffer Updater | Low        | Simple broadcast subscriber |
| Lexer                  | Medium     | Token state machine         |
| Parser                 | High       | Recursive descent           |
| Interpreter            | High       | Scope + async builtins      |

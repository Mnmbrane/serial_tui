# SerialTUI Kanban

## Backlog

### Foundation

- [ ] Project setup (Cargo.toml, directory structure, mod.rs files)
- [ ] Error handling setup (color-eyre, thiserror, custom error types)
- [ ] Basic main.rs with tokio runtime

### Config Module

- [ ] Define `PortConfig` struct with serde
- [ ] Define `AppConfig` struct (defaults + ports)
- [ ] Define `MacroConfig` struct
- [ ] Config file parser (load config.toml)
- [ ] Config validation (unique names, unique paths)
- [ ] Config save functionality
- [ ] Default value generation (random unique colors)
- [ ] Macro file parser (load macros.toml)

### Serial Module

- [ ] Define `SerialMessage` struct
- [ ] Define `SerialCommand` enum
- [ ] Port connection logic (tokio-serial)
- [ ] Line buffering (configurable line ending)
- [ ] Single port read task (async)
- [ ] Single port write handling
- [ ] Broadcast channel setup (fan-out)
- [ ] Auto-reconnect logic with backoff
- [ ] Port manager (spawn/despawn port tasks)
- [ ] Binary data handling

### Logger Module

- [ ] Log formatter (`<timestamp>[port_name] data`)
- [ ] Combined log writer (all.log)
- [ ] Per-port log writer (<port>.log)
- [ ] Async file writing (non-blocking)
- [ ] Log directory creation

### UI - Core

- [ ] Basic ratatui app loop
- [ ] Main layout (top/middle/bottom split)
- [ ] App mode state machine (Normal, Debug, Command)
- [ ] Focus management (display vs input box)
- [ ] Keyboard event routing

### UI - Config Bar (Top)

- [ ] Port dropdown widget
- [ ] Filter dropdown (hide ports from display)
- [ ] "Add Port" button
- [ ] "Run Script" dropdown
- [ ] Layout and styling

### UI - Display (Middle)

- [ ] Display buffer (VecDeque<SerialMessage>)
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
- [ ] Insert mode
- [ ] Normal mode (vim commands)
- [ ] Cursor movement
- [ ] Basic editing (backspace, delete)
- [ ] Command history
- [ ] `Ctrl+R` reverse search (display buffer)

### UI - Port Selector (Bottom Left)

- [ ] Checkbox list widget
- [ ] Scrollable container
- [ ] Toggle port selection
- [ ] Visual styling (checked/unchecked)

### UI - Popups

- [ ] Popup overlay system
- [ ] Add Port popup form
- [ ] Color picker popup
- [ ] Exit confirmation popup
- [ ] Popup keyboard handling

### UI - Notifications

- [ ] Notification struct (message, type, duration)
- [ ] Duration calculation (char count + 1s)
- [ ] Notification queue (stacking)
- [ ] Top-right rendering
- [ ] Auto-dismiss timer
- [ ] Notification types (info, error, success)

### UI - Debug Screen

- [ ] Debug screen overlay
- [ ] `:debug` toggle command
- [ ] Debug log buffer (eprint capture)
- [ ] `:memory` command output
- [ ] `:thread` command output
- [ ] Scroll within debug screen

### Script Engine - Lexer

- [ ] Token definitions
- [ ] Number literals (f64)
- [ ] String literals (with escapes)
- [ ] Boolean literals
- [ ] Identifiers
- [ ] Keywords (let, if, elif, else, while, for, in, fn, return, true, false)
- [ ] Operators (+, -, *, /, %, ==, !=, <, >, <=, >=, &&, ||, !)
- [ ] Punctuation (; , ( ) [ ] { } ..)
- [ ] Raw string literals (r"...")
- [ ] Hex literals (0x...)
- [ ] Comments (// and /**/)
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
- [ ] Variable storage
- [ ] Expression evaluation
- [ ] Number operations
- [ ] String operations
- [ ] Boolean operations
- [ ] Array operations
- [ ] Control flow (if/elif/else)
- [ ] Loop execution (while, for)
- [ ] Function calls (user-defined)
- [ ] Function return values
- [ ] Built-in: `sendstr(ports, string)`
- [ ] Built-in: `sendbin(ports, hex)`
- [ ] Built-in: `wait(seconds)`
- [ ] Built-in: `waitstr(ports, regex, timeout)`
- [ ] Script abort on waitstr timeout
- [ ] Async execution (non-blocking wait)
- [ ] Error handling and reporting

### App Integration

- [ ] App state struct
- [ ] Channel wiring (serial <-> ui <-> script)
- [ ] Command parsing (`:q`, `:w`, `:run`, etc.)
- [ ] Script execution management
- [ ] Macro execution
- [ ] Graceful shutdown

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

### M1: Basic Serial Connection

- Project setup
- Config module (load only)
- Serial module (single port read)
- Logger module (combined log)
- Basic main loop

### M2: Minimal TUI

- Basic ratatui loop
- Display buffer rendering
- Input box (basic)
- Send to single port

### M3: Multi-Port Support

- Multiple port tasks
- Broadcast channel
- Port selector checkboxes
- Interleaved display with colors
- Per-port logging

### M4: Vim Navigation

- Display navigation (j/k/gg/G)
- Search (/)
- Yank (y/yy)
- Preview panel

### M5: Config UI

- Config bar dropdowns
- Add port popup
- Color picker popup
- Filter dropdown
- Save config

### M6: Script Engine

- Lexer
- Parser
- Interpreter (basic)
- Built-in functions
- Script execution from UI

### M7: Polish

- Notifications
- Debug screen
- Auto-reconnect
- Error handling
- Commands (:q, :w, :debug, etc.)

---

## Notes

### Priority Order

1. Foundation + Config + Serial (get data flowing)
2. Basic UI (see the data)
3. Vim navigation (interact with data)
4. Multi-port (scale up)
5. Script engine (automation)
6. Polish (production ready)

### Dependencies

```
Config ─────────────────────────┐
                                ▼
Serial ──────────────────────► App Integration
                                ▲
Logger ─────────────────────────┤
                                │
UI (all components) ────────────┤
                                │
Script Engine ──────────────────┘
```

### Complexity Estimates

| Component | Complexity | Notes |
|-----------|------------|-------|
| Config | Low | Serde does heavy lifting |
| Serial | Medium | Async + reconnect logic |
| Logger | Low | Straightforward file I/O |
| UI Core | Medium | Layout + state management |
| Vim Nav | Medium | Multiple modes + state |
| Vim Search | Medium | Regex + highlighting |
| Popups | Medium | Overlay system |
| Notifications | Low | Timer + queue |
| Lexer | Medium | Token state machine |
| Parser | High | Recursive descent + precedence |
| Interpreter | High | Scope + async builtins |

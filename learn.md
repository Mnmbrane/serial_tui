# SerialTUI Learning Guide

What to learn before starting implementation.

---

## 1. Rust Fundamentals

### Must Know

- [ ] Ownership, borrowing, lifetimes
- [ ] Error handling (`Result`, `?` operator, `thiserror`)
- [ ] Enums with data (`enum Command { Send { data: Vec<u8> } }`)
- [ ] Pattern matching (`match`, `if let`)
- [ ] Traits (`impl Trait`, trait bounds, `dyn Trait`)
- [ ] Generics
- [ ] Closures (`|x| x + 1`, `move` keyword)
- [ ] Iterators (`.map()`, `.filter()`, `.collect()`)

### Rust Resources

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)

---

## 2. Async Rust (Tokio)

### Async Concepts

- [ ] `async`/`await` syntax
- [ ] Futures and how they work
- [ ] Tokio runtime (`#[tokio::main]`, `Runtime::new()`)
- [ ] Spawning tasks (`tokio::spawn`)
- [ ] `select!` macro (wait on multiple futures)
- [ ] Cancellation (dropping futures, `CancellationToken`)

### Channels (Critical for this project)

- [ ] `tokio::sync::mpsc` - Multi-producer, single-consumer (commands)
- [ ] `tokio::sync::broadcast` - Multi-producer, multi-consumer (serial messages)
- [ ] `tokio::sync::oneshot` - Single-use channel (script abort)
- [ ] Channel capacity and backpressure

### Synchronization

- [ ] `Arc<Mutex<T>>` for shared state
- [ ] `tokio::sync::Mutex` vs `std::sync::Mutex`
- [ ] When to use `RwLock` vs `Mutex`

### Timers

- [ ] `tokio::time::sleep`
- [ ] `tokio::time::timeout`
- [ ] `tokio::time::interval`

### Tokio Resources

- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Async Book](https://rust-lang.github.io/async-book/)

### Tokio Practice Exercise

```rust
// Create two tasks that communicate via mpsc channel
// Task 1: Sends numbers 1-10
// Task 2: Receives and prints them
// Use select! to also listen for a shutdown signal
```

---

## 3. Ratatui (TUI Framework)

### Ratatui Core Concepts

- [ ] Terminal setup/restore (`ratatui::init()`, `ratatui::restore()`)
- [ ] The render loop pattern
- [ ] `Frame` and drawing
- [ ] `Rect` (areas/regions)
- [ ] Immediate mode rendering (redraw everything each frame)

### Layout System

- [ ] `Layout::default().direction().constraints().split()`
- [ ] `Constraint::Percentage`, `Constraint::Length`, `Constraint::Min`, `Constraint::Max`
- [ ] Nested layouts

### Built-in Widgets

- [ ] `Paragraph` - Text display
- [ ] `Block` - Borders and titles
- [ ] `List` - Scrollable lists
- [ ] `Table` - Tabular data
- [ ] `Tabs` - Tab bar
- [ ] `Gauge` - Progress bar

### Styling

- [ ] `Style::default().fg().bg().add_modifier()`
- [ ] `Color` enum
- [ ] `Modifier` (Bold, Italic, Underline)
- [ ] `Span` and `Line` for styled text

### Custom Widgets

- [ ] `impl Widget for MyWidget`
- [ ] `impl StatefulWidget for MyWidget`

### Ratatui Resources

- [Ratatui Book](https://ratatui.rs/)
- [Ratatui Examples](https://github.com/ratatui-org/ratatui/tree/main/examples)

### Ratatui Practice Exercise

```rust
// Create a TUI with:
// - Top: Title bar
// - Middle: Scrollable list of items
// - Bottom: Input box
// Handle 'q' to quit, arrow keys to scroll
```

---

## 4. Crossterm (Terminal Backend)

### Event Handling

- [ ] `crossterm::event::read()` - Blocking read
- [ ] `crossterm::event::poll()` - Non-blocking check
- [ ] `Event::Key(KeyEvent)` - Keyboard events
- [ ] `KeyCode`, `KeyModifiers`
- [ ] `Event::Mouse` - Mouse events (if needed)
- [ ] `Event::Resize` - Terminal resize

### Terminal Control

- [ ] Raw mode (what it is, why it's needed)
- [ ] Alternate screen
- [ ] Cursor visibility

### Crossterm Resources

- [Crossterm Docs](https://docs.rs/crossterm/latest/crossterm/)

### Crossterm Practice Exercise

```rust
// Print which key the user pressed
// Handle Ctrl+C to exit
// Handle arrow keys, Enter, Escape
```

---

## 5. Serial Communication (tokio-serial)

### Serial Concepts

- [ ] What is a serial port (UART, baud rate, data bits, parity, stop bits)
- [ ] Serial port paths (`/dev/ttyUSB0`, `COM1`)
- [ ] Flow control (hardware RTS/CTS, software XON/XOFF)
- [ ] Line endings (`\n` vs `\r\n`)

### tokio-serial API

- [ ] `tokio_serial::new(path, baud_rate)`
- [ ] `SerialPortBuilderExt::open_native_async()`
- [ ] `AsyncReadExt::read()` - Read bytes
- [ ] `AsyncWriteExt::write_all()` - Write bytes
- [ ] `BufReader` for line-by-line reading

### Serial Resources

- [tokio-serial Docs](https://docs.rs/tokio-serial/latest/tokio_serial/)
- [Serial Port Basics](https://learn.sparkfun.com/tutorials/serial-communication)

### Serial Practice Exercise

```rust
// Open a serial port
// Read lines in a loop, print each one with timestamp
// Handle disconnection gracefully
```

---

## 6. Configuration (Serde + TOML)

### Serde

- [ ] `#[derive(Serialize, Deserialize)]`
- [ ] `#[serde(default)]` - Default values
- [ ] `#[serde(default = "function_name")]` - Custom default
- [ ] `#[serde(rename = "name")]` - Rename fields
- [ ] `#[serde(skip)]` - Skip fields

### TOML

- [ ] Basic syntax (tables, arrays of tables, inline tables)
- [ ] `toml::from_str()` - Parse
- [ ] `toml::to_string_pretty()` - Serialize

### Config Resources

- [Serde Guide](https://serde.rs/)
- [TOML Spec](https://toml.io/)

### Config Practice Exercise

```rust
// Define a Config struct with nested Port struct
// Load from file, modify, save back
// Handle missing file (create default)
```

---

## 7. Regex (for search and waitstr)

### Regex Concepts

- [ ] Basic regex syntax (`.*`, `\d+`, `[a-z]`, `^`, `$`)
- [ ] Capture groups `(...)`
- [ ] Non-capturing groups `(?:...)`
- [ ] Alternation `a|b`

### Rust Regex Crate

- [ ] `Regex::new(pattern)`
- [ ] `regex.is_match(text)`
- [ ] `regex.find(text)` - First match
- [ ] `regex.captures(text)` - Capture groups
- [ ] Compile once, use many times (avoid recompiling in loops)

### Regex Resources

- [Regex Crate Docs](https://docs.rs/regex/latest/regex/)
- [Regex101](https://regex101.com/) - Online tester

### Regex Practice Exercise

```rust
// Match "OK" or "ERROR" in serial output
// Extract number from "Temperature: 25.5C"
// Handle invalid regex gracefully
```

---

## 8. Algorithms and Data Structures

### Line Buffering

```text
Input stream: H e l l o \n W o r l d \n
                      ^           ^
                   emit        emit

Algorithm:
1. Read bytes into buffer
2. Scan for delimiter
3. If found: emit line (without delimiter), keep remainder
4. If not found: keep buffering
```

### Exponential Backoff

```text
delay = min(max_delay, min_delay * (multiplier ^ attempts))

Example (min=1s, max=8s, mult=2):
Attempt 1: 1s
Attempt 2: 2s
Attempt 3: 4s
Attempt 4: 8s
Attempt 5: 8s (capped)
```

### Scroll Position Management

```text
total_lines = 1000
viewport_height = 20
scroll_offset = 500  // First visible line

// Visible range: lines 500-519
// scroll_offset bounds: 0 to (total_lines - viewport_height)

// Scroll down: scroll_offset = min(scroll_offset + 1, max_offset)
// Scroll up: scroll_offset = max(scroll_offset - 1, 0)
// Jump to bottom: scroll_offset = max_offset
// Jump to top: scroll_offset = 0
```

### Notification Timer

```text
duration_secs = (char_count / chars_per_second) + 1.0

// Average reading speed: ~15-20 chars/second
// "Connected to GPS" (16 chars) -> ~2 seconds display time
```

---

## 9. Script Engine (Interpreter)

### Lexer (Tokenizer)

- [ ] What is a token (Number, String, Identifier, Keyword, Operator, etc.)
- [ ] State machine approach
- [ ] Handling string escapes (`\n`, `\"`, `\\`)
- [ ] Tracking line/column for errors

### Parser

- [ ] Recursive descent parsing
- [ ] Operator precedence (Pratt parsing / precedence climbing)
- [ ] Abstract Syntax Tree (AST) structure
- [ ] Error recovery strategies

### Interpreter

- [ ] Tree-walking interpreter
- [ ] Environment/scope stack
- [ ] Variable binding and lookup
- [ ] Function calls and returns
- [ ] Control flow (if/while/for)

### Interpreter Resources

- [Crafting Interpreters](https://craftinginterpreters.com/) - Excellent free book
- [Writing An Interpreter In Go](https://interpreterbook.com/) - Concepts apply to Rust

### Interpreter Practice Exercise

```rust
// Start simple:
// 1. Lexer that tokenizes: let x = 1 + 2;
// 2. Parser that builds AST for above
// 3. Interpreter that evaluates to x = 3
// Then gradually add: if, while, functions
```

---

## 10. Clipboard (arboard)

### Clipboard API

- [ ] `Clipboard::new()`
- [ ] `clipboard.set_text(String)`
- [ ] `clipboard.get_text()`

### Clipboard Resources

- [arboard Docs](https://docs.rs/arboard/latest/arboard/)

---

## 11. Error Handling (color-eyre + thiserror)

### thiserror (Defining Errors)

```rust
#[derive(Debug, thiserror::Error)]
pub enum SerialError {
    #[error("Port not found: {0}")]
    PortNotFound(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(#[from] std::io::Error),
}
```

### color-eyre (Pretty Errors)

```rust
use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    // Your code...
    Ok(())
}
```

### Error Handling Resources

- [thiserror Docs](https://docs.rs/thiserror/latest/thiserror/)
- [color-eyre Docs](https://docs.rs/color-eyre/latest/color_eyre/)

---

## 12. Suggested Learning Order

### Phase 1: Foundations (1-2 weeks)

- Rust fundamentals (if needed)
- Async Rust + Tokio basics
- Channels (mpsc, broadcast)

### Phase 2: TUI + Serial (1-2 weeks)

- Crossterm event handling
- Ratatui basics
- tokio-serial basics

### Phase 3: Integration (1 week)

- Serde + TOML config
- Regex basics
- Error handling

### Phase 4: Script Engine (2-4 weeks)

- Crafting Interpreters (chapters 4-10)
- Build lexer
- Build parser
- Build interpreter

---

## 13. Mini Projects Before Starting

### Project 1: Async Echo Server

- Two tasks communicating via channels
- One reads stdin, sends to channel
- One receives from channel, prints with timestamp
- Graceful shutdown with Ctrl+C

### Project 2: Simple TUI Counter

- Display a number
- Up/Down arrows to increment/decrement
- 'q' to quit
- Proper terminal setup/restore

### Project 3: Serial Monitor (Minimal)

- Open one serial port
- Display incoming lines
- Text input to send data
- No fancy features, just the basics

### Project 4: Calculator Interpreter

- Lexer: numbers, +, -, *, /, (, )
- Parser: expression tree with precedence
- Interpreter: evaluate and print result
- This teaches 80% of what you need for .stui scripts

---

## 14. Reference Implementations

### Similar Projects to Study

- [serialport-rs examples](https://github.com/serialport/serialport-rs/tree/main/examples)
- [ratatui examples](https://github.com/ratatui-org/ratatui/tree/main/examples)
- [bottom](https://github.com/ClementTsang/bottom) - TUI system monitor in Rust
- [gitui](https://github.com/extrawurst/gitui) - TUI git client in Rust

### Interpreter References

- [Lox in Rust](https://github.com/tdp2110/crafting-interpreters-rs)
- [monkey-rust](https://github.com/Rydgel/monkey-rust)

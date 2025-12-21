# SerialTUI Learning Guide

Prerequisites for working on this project.

---

## 1. Rust Fundamentals

- [ ] Ownership, borrowing, lifetimes
- [ ] Error handling (`Result`, `?`, `thiserror`)
- [ ] Enums with data
- [ ] Pattern matching
- [ ] Traits and generics
- [ ] Closures and iterators

**Resources:** [The Rust Book](https://doc.rust-lang.org/book/)

---

## 2. Async Rust (Tokio)

### Core Concepts
- [ ] `async`/`await` syntax
- [ ] Tokio runtime (`#[tokio::main]`)
- [ ] Spawning tasks (`tokio::spawn`)
- [ ] `select!` macro

### Channels (Critical)
- [ ] `mpsc` - multi-producer, single-consumer
- [ ] `broadcast` - multi-producer, multi-consumer
- [ ] `oneshot` - single-use

### Synchronization
- [ ] `Arc<Mutex<T>>` for shared state
- [ ] `tokio::sync::Mutex` vs `std::sync::Mutex`

### Timers
- [ ] `tokio::time::sleep`
- [ ] `tokio::time::timeout`

**Resources:** [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

---

## 3. Ratatui (TUI)

- [ ] Terminal setup/restore
- [ ] Render loop pattern
- [ ] Layout system (`Constraint`, `Direction`)
- [ ] Widgets (`Paragraph`, `Block`, `List`)
- [ ] Styling (`Style`, `Color`, `Span`)

**Resources:** [Ratatui Book](https://ratatui.rs/)

---

## 4. Crossterm (Terminal Backend)

- [ ] Event polling and reading
- [ ] `KeyCode`, `KeyModifiers`
- [ ] Raw mode and alternate screen

---

## 5. Serial Communication

- [ ] UART basics (baud rate, data bits, parity, stop bits)
- [ ] Port paths (`/dev/ttyUSB0`, `COM1`)
- [ ] Line endings (`\n` vs `\r\n`)
- [ ] `tokio-serial` API

**Resources:** [tokio-serial Docs](https://docs.rs/tokio-serial/)

---

## 6. Serde + TOML

- [ ] `#[derive(Serialize, Deserialize)]`
- [ ] `#[serde(default)]` for optional fields
- [ ] TOML syntax

---

## 7. Regex

- [ ] Basic patterns (`.*`, `\d+`, `[a-z]`)
- [ ] `Regex::new()`, `is_match()`, `find()`

---

## 8. Script Engine

- [ ] Lexer: source -> tokens
- [ ] Parser: tokens -> AST (recursive descent)
- [ ] Interpreter: tree-walking execution

**Resources:** [Crafting Interpreters](https://craftinginterpreters.com/)

---

## Key Algorithms

### Exponential Backoff
```
delay = min(max_delay, min_delay * 2^attempts)
Example: 1s -> 2s -> 4s -> 8s (capped)
```

### Scroll Position
```
visible_start = total - viewport_height - scroll_offset
visible_end = total - scroll_offset
```

### Notification Duration
```
duration = (char_count / 15) + 1 second
```

---

## Practice Projects

1. **Async Echo** - Two tasks, mpsc channel, graceful shutdown
2. **TUI Counter** - Up/down arrows, quit with 'q'
3. **Serial Monitor** - One port, display lines, send text
4. **Calculator** - Lexer + parser + interpreter for `1 + 2 * 3`

---

## Reference Projects

- [ratatui examples](https://github.com/ratatui-org/ratatui/tree/main/examples)
- [bottom](https://github.com/ClementTsang/bottom) - TUI system monitor
- [gitui](https://github.com/extrawurst/gitui) - TUI git client

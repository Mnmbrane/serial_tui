# SerialTUI Architecture

## Overview

SerialTUI is a terminal UI for multi-port serial communication with vim-style navigation and scripting.

```
+------------------+     +-----------+     +----------------+
|     UI Task      | --> | Command   | --> | Serial Handler |
|    (ratatui)     |     | Dispatcher|     | (per-port tasks)
+------------------+     +-----------+     +-------+--------+
        ^                                          |
        |              broadcast channel           |
        +------------------------------------------+
```

---

## Directory Structure

```
src/
├── main.rs           # Entry point, spawns all tasks
├── app.rs            # AppState (shared state)
├── error.rs          # Error types
├── config/           # Config loading/saving
├── serial/           # Port tasks, messages, commands
├── ui/               # Widgets, layout, vim modes
├── logger/           # File writers
├── notification/     # Toast system
└── script/           # Lexer, parser, interpreter

config/
├── config.toml       # Port definitions
└── macros.toml       # Keybind macros

scripts/              # User .stui scripts
logs/                 # Output logs
```

---

## Data Flow

### Channels

| Channel | Type | From | To | Purpose |
|---------|------|------|-----|---------|
| app_command | mpsc | UI | Dispatcher | User actions (quit, send, run script) |
| serial_command | mpsc | Dispatcher, Script | Serial Handler | Port operations |
| serial_broadcast | broadcast | Serial Handler | Display, Logger, Script | All received data |
| notification | mpsc | Any | Notification System | Toast messages |
| script_abort | oneshot | Dispatcher | Script Engine | Stop signal |

### Flow Diagram

```
USER INPUT
    |
    v
+-------------------+
| UI: handle key    |
| - vim nav: local  |
| - :cmd/text: send |
+-------------------+
    |
    | AppCommand (SendText, RunScript, Quit, ...)
    v
+-------------------+
| DISPATCHER        |
| - translate cmds  |
| - spawn scripts   |
| - update state    |
+-------------------+
    |
    | SerialCommand (Send, Connect, Disconnect)
    v
+-------------------+
| SERIAL HANDLER    |
| - manage ports    |
| - route writes    |
+-------------------+
    |
    | SerialMessage (per port task)
    v
+-------------------+
| BROADCAST         |----+----+----+
+-------------------+    |    |    |
                         v    v    v
                    Display Logger Script
                    Updater        (waitstr)
```

---

## Key Components

### Serial Port Task

```
PSEUDOCODE: run_port_task(config, broadcast_tx, write_rx)

    backoff_delay = 1 second

    loop forever:
        try:
            port = open_serial(config.path, config.baud_rate)
            backoff_delay = 1 second  // reset on success

            loop:
                select:
                    line = read_line(port):
                        msg = SerialMessage {
                            timestamp: now(),
                            port_name: config.name,
                            data: line
                        }
                        broadcast_tx.send(msg)

                    data = write_rx.receive():
                        port.write(data)

        on error:
            notify("Port disconnected, retrying...")
            sleep(backoff_delay)
            backoff_delay = min(backoff_delay * 2, 8 seconds)
```

### Display Buffer Updater

```
PSEUDOCODE: display_updater(state, broadcast_rx)

    loop:
        msg = broadcast_rx.receive()

        lock state:
            state.display_buffer.push(msg)

            if state.display_buffer.len > 10000:
                state.display_buffer.pop_front()
```

### Command Dispatcher

```
PSEUDOCODE: command_dispatcher(app_rx, serial_tx, notif_tx, state)

    loop:
        cmd = app_rx.receive()

        match cmd:
            Quit:
                state.running = false
                break

            SendText(ports, text):
                data = text + line_ending
                serial_tx.send(Send { ports, data })

            RunScript(path):
                spawn script_engine(path, serial_tx, broadcast, abort_rx)
                notif_tx.send("Script started")

            StopScript:
                abort_tx.send(())
                notif_tx.send("Script aborted")

            ClearDisplay:
                state.display_buffer.clear()
```

### Notification System

```
PSEUDOCODE: notification_system(notif_rx, state)

    loop:
        msg = notif_rx.receive()

        duration = (msg.length / 15) + 1 second  // reading speed

        notification = Notification {
            message: msg,
            created_at: now(),
            duration: duration
        }

        state.notifications.push(notification)
```

### UI Render Loop

```
PSEUDOCODE: ui_loop(terminal, state)

    loop:
        lock state:
            terminal.draw(render_ui(state))

            if not state.running:
                break
        // lock released

        if poll_input(16ms):  // ~60 FPS
            event = read_input()
            handle_key(event, state)
```

---

## Vim Mode State Machine

```
STATES: Normal, Insert, Command, Search
FOCUS:  Display, InputBox

TRANSITIONS:
    Display + Normal:
        'i'     -> InputBox + Insert
        ':'     -> Command mode
        '/'     -> Search mode
        'j'     -> scroll down
        'k'     -> scroll up
        'gg'    -> jump to top
        'G'     -> jump to bottom
        Ctrl+d  -> half-page down
        Ctrl+u  -> half-page up
        'y'     -> yank selection
        'n'     -> next search match
        'N'     -> prev search match

    InputBox + Insert:
        Esc     -> InputBox + Normal
        Enter   -> send text, clear input
        chars   -> append to input

    InputBox + Normal:
        'i'     -> Insert mode
        Esc     -> Display + Normal
        'h'/'l' -> cursor left/right
        'w'/'b' -> word forward/back
        'x'     -> delete char

    Command:
        Enter   -> execute command, return to Normal
        Esc     -> cancel, return to Normal

    Search:
        Enter   -> execute search, return to Normal
        Esc     -> cancel, return to Normal
```

---

## Script Engine (.stui)

### Pipeline

```
Source Code -> Lexer -> Tokens -> Parser -> AST -> Interpreter -> Execution
```

### Lexer

```
INPUT:  "let x = 1 + 2;"
OUTPUT: [Let, Ident("x"), Equals, Number(1), Plus, Number(2), Semicolon]
```

### Parser (Recursive Descent)

```
INPUT:  [Let, Ident("x"), Equals, Number(1), Plus, Number(2), Semicolon]
OUTPUT: LetStmt {
            name: "x",
            value: BinaryOp {
                left: Number(1),
                op: Plus,
                right: Number(2)
            }
        }
```

### Interpreter

```
PSEUDOCODE: interpret(ast, env, serial_tx, broadcast_rx)

    for stmt in ast:
        match stmt:
            Let { name, value }:
                result = eval_expr(value, env)
                env.set(name, result)

            If { cond, then, else }:
                if eval_expr(cond, env) is truthy:
                    interpret(then, env)
                else if else exists:
                    interpret(else, env)

            While { cond, body }:
                while eval_expr(cond, env) is truthy:
                    interpret(body, env)

            Expr(call):
                if call is builtin:
                    execute_builtin(call, env, serial_tx, broadcast_rx)
                else:
                    call user function
```

### Built-in Functions

| Function | Description | Example |
|----------|-------------|---------|
| `sendstr(ports, text)` | Send string | `sendstr(["GPS"], "PING\n")` |
| `sendbin(ports, hex)` | Send binary | `sendbin(["Motor"], "0x01020304")` |
| `wait(seconds)` | Pause | `wait(0.5)` |
| `waitstr(ports, regex, timeout)` | Wait for pattern | `waitstr(["GPS"], r"OK\|ACK", 5.0)` |

---

## Key Data Types

### SerialMessage

```
timestamp   : DateTime     // when received
port_name   : String       // display name ("GPS")
port_path   : String       // system path ("/dev/ttyUSB0")
data        : String       // line content (no newline)
```

### PortConfig

```
name        : String       // unique display name
path        : String       // unique system path
baud_rate   : u32          // default: 115200
line_ending : String       // default: "\n"
color       : String       // hex color for UI
```

### AppState

```
port_configs   : Map<name, PortConfig>
display_buffer : Queue<SerialMessage>   // capped at 10000
notifications  : Queue<Notification>
scroll_offset  : usize
input_text     : String
vim_mode       : VimMode
focus          : Focus
running        : bool
```

---

## Config File Format

### config/config.toml

```toml
[[port]]
name = "GPS"
path = "/dev/ttyUSB0"
baud_rate = 115200
color = "#FF5733"

[[port]]
name = "Motor"
path = "/dev/ttyUSB1"
baud_rate = 9600
line_ending = "\r\n"
```

### config/macros.toml

```toml
[[macro]]
name = "ping_all"
key = "F1"
command = "sendstr([\"GPS\", \"Motor\"], \"PING\\n\")"
```

---

## Log Format

```
<2025-01-15 14:32:01.123>[GPS] NMEA: $GPGGA,123456...
<2025-01-15 14:32:01.456>[Motor] RPM: 1500
```

Files:

- `logs/all.log` - combined (all ports)
- `logs/<port_name>.log` - per-port

---

## Error Handling

### Strategy

- Config errors: notify user, continue with defaults
- Serial errors: auto-reconnect with backoff
- Script errors: abort script, notify user
- Channel errors: log and continue (indicates shutdown)

### Backoff Pattern

```
Attempt 1: wait 1s
Attempt 2: wait 2s
Attempt 3: wait 4s
Attempt 4+: wait 8s (capped)
On success: reset to 1s
```

---

## UI Layout

```
+---------------------------------------------------------------------+
| [Port v] [Filter v] [Add Port] [Run Script v]         Notifications |
+---------------------------------------------------------------------+
| <14:32:01>[GPS] data line 1...                                      |
| <14:32:02>[Motor] data line 2...                                    |
| <14:32:03>[GPS] data line 3...                                      |
|                                                                     |
| ~ vim: j/k scroll, /search, y yank                                  |
+---------------------------------------------------------------------+
| PREVIEW: selected line with word wrap                               |
+---------------------------------------------------------------------+
| [x] GPS   | > input text here_                                      |
| [ ] Motor |                                                         |
+-----------+---------------------------------------------------------+
```

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| tokio | Async runtime |
| ratatui | TUI framework |
| crossterm | Terminal backend |
| tokio-serial | Async serial ports |
| serde + toml | Config parsing |
| chrono | Timestamps |
| regex | Search + waitstr |
| arboard | Clipboard |
| thiserror | Error types |
| color-eyre | Error display |
| rand | Random colors |

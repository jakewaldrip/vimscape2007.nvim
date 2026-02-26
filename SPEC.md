# Vimscape2007 - Technical Specification

This document provides a comprehensive specification for the Vimscape2007 Neovim plugin, a gamification system that tracks Vim keystrokes and awards experience points in the style of RuneScape (2007 era).

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Data Flow](#data-flow)
4. [Lua Frontend](#lua-frontend)
5. [Rust Backend](#rust-backend)
6. [Lexer Specification](#lexer-specification)
7. [Token Definitions](#token-definitions)
8. [Skills System](#skills-system)
9. [Experience & Leveling](#experience--leveling)
10. [Database Schema](#database-schema)
11. [User Interface](#user-interface)
12. [Configuration](#configuration)

---

## Overview

Vimscape2007 is a hybrid Lua/Rust Neovim plugin that:

1. Captures keystrokes from Neovim in normal mode
2. Batches and sends them to a Rust backend
3. Lexes the keystroke stream into semantic Vim command tokens
4. Maps tokens to skills and awards experience points
5. Persists skill data to SQLite
6. Notifies users of level-ups via Neovim notifications

The plugin gamifies Vim usage by tracking 11 distinct skills, each leveling from 1-99 using a RuneScape-inspired exponential XP curve.

---

## Architecture

```
+------------------------------------------------------------------+
|                           Neovim                                 |
|  +------------------------------------------------------------+  |
|  |                      Lua Frontend                          |  |
|  |                                                            |  |
|  |  init.lua ──> keys.lua ──> vimscape_backend.so            |  |
|  |      │            │               │                        |  |
|  |  Commands    Keystroke       Rust FFI                      |  |
|  |  & UI        Capture        (nvim-oxi)                     |  |
|  +------------------------------------------------------------+  |
|                              │                                   |
|  +------------------------------------------------------------+  |
|  |                      Rust Backend                          |  |
|  |                                                            |  |
|  |  api.rs ──> lexer.rs ──> parse_utils.rs ──> skills.rs     |  |
|  |     │                                           │          |  |
|  |  db.rs (SQLite)                    levels.rs (XP curve)   |  |
|  +------------------------------------------------------------+  |
+------------------------------------------------------------------+
```

### Component Responsibilities

| Component | Responsibility |
|-----------|----------------|
| `init.lua` | Plugin entry point, commands, UI windows |
| `keys.lua` | Keystroke capture, sanitization, batching |
| `globals.lua` | Runtime state management |
| `config.lua` | Plugin configuration with defaults |
| `window_config.lua` | Floating window configurations |
| `api.rs` | Public API exposed to Lua |
| `lexer.rs` | Keystroke stream tokenization |
| `token.rs` | Token type definitions |
| `parse_utils.rs` | Token-to-skill mapping with XP values |
| `skills.rs` | Skill enum and utilities |
| `levels.rs` | XP curve and level calculations |
| `db.rs` | SQLite persistence layer |
| `skill_data.rs` | Data formatting for display |

---

## Data Flow

### Keystroke Processing Flow

```
1. User presses keys in Neovim (normal mode)
              │
              ▼
2. keys.lua captures via vim.on_key()
              │
              ▼
3. sanitize_key() filters/translates special keys
              │
              ▼
4. Keys accumulated in globals.typed_letters
              │
              ▼
5. When batch_size reached, concatenate and send to Rust
              │
              ▼
6. api.process_batch() receives string
              │
              ▼
7. Lexer tokenizes input stream
              │
              ▼
8. parse_utils maps tokens to skills with XP
              │
              ▼
9. XP accumulated by skill, written to database
              │
              ▼
10. Level changes calculated and notified
```

### Key Sanitization

The Lua frontend translates Neovim's key representations into a format the Rust lexer can parse:

| Neovim Key | Sanitized Output |
|------------|------------------|
| `<CR>` | `\|enter\|` |
| `<Tab>` | `\|tab\|` |
| `<BS>` | `\|backspace\|` |
| `<Del>` | `\|delete\|` |
| `<Space>` | `\|space\|` |
| `<Esc>` | `\|escape\|` |
| `<Up>` | `\|up\|` |
| `<Down>` | `\|down\|` |
| `<Left>` | `\|left\|` |
| `<Right>` | `\|right\|` |
| `<Home>` | `\|home\|` |
| `<End>` | `\|end\|` |
| `<PageUp>` | `\|pageup\|` |
| `<PageDown>` | `\|pagedown\|` |
| `<Insert>` | `\|insert\|` |
| `<C-X>` | `<C-X>` (preserved) |
| Mouse events | Skipped |
| Terminal codes | Skipped |
| Insert mode | Skipped |
| `<Cmd>` | Skipped |
| Extended function keys (F13-F37) | Skipped |

---

## Lua Frontend

### Module: `vimscape2007/init.lua`

**Exported Functions:**

| Function | Description |
|----------|-------------|
| `setup(opts)` | Initialize plugin with optional config overrides |
| `toggle()` | Toggle keystroke recording on/off |
| `show_data()` | Display skills window |
| `show_details(skill_name)` | Display details for a specific skill |
| `create_user_commands()` | Register `:Vimscape` command |

**User Commands:**

| Command | Description |
|---------|-------------|
| `:Vimscape stats` | Open skills display window |
| `:Vimscape details` | Show details for skill under cursor |
| `:Vimscape toggle` | Toggle recording on/off |

**Window Keymaps (Stats Window):**

| Key | Action |
|-----|--------|
| `q` | Close window |
| `d` | Show details for skill under cursor |

### Module: `keys.lua`

**Functions:**

| Function | Description |
|----------|-------------|
| `sanitize_key(key)` | Filter and translate special keys |
| `record_keys()` | Set up vim.on_key handler for keystroke capture |

**Behavior:**
- Ignores insert mode keystrokes
- Filters out mouse events and terminal escape codes
- Translates special keys to pipe-delimited format
- Accumulates keys until `batch_size` threshold
- Calls `process_batch()` when threshold reached

### Module: `globals.lua`

**State Variables:**

| Variable | Type | Description |
|----------|------|-------------|
| `active` | boolean | Whether recording is enabled |
| `typed_letters` | table | Accumulated keystrokes |

### Module: `config.lua`

**Configuration Options:**

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `db_path` | string | Plugin directory | Path to SQLite database |
| `batch_size` | number | 50 | Keys before processing |
| `log_level` | string | "INFO" | Minimum notification level |
| `batch_notify` | boolean | false | Notify on batch processing |

---

## Rust Backend

### Module: `lib.rs`

Plugin entry point using `#[nvim_oxi::plugin]` macro. Exports a Dictionary with four functions to Lua.

### Module: `api.rs`

**Exported Functions:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `process_batch` | `(input: String, db_path: String) -> bool` | Lex input, award XP, update DB |
| `get_user_data` | `(col_len: i32, db_path: String) -> Vec<String>` | Get formatted skill display |
| `setup_tables` | `(db_path: String)` | Initialize database schema |
| `get_skill_details` | `(skill_name: String, db_path: String) -> Vec<String>` | Get single skill details |

**process_batch Algorithm:**

```
1. Create Lexer from input string
2. Initialize HashMap<Skill, i32> for XP accumulation
3. For each token from lexer:
   a. Call token_to_skill() to get Option<Skill>
   b. If Some(skill), add skill's XP to accumulator
4. Read current skill data from database
5. For each accumulated skill:
   a. Write new XP to database
6. Calculate level changes
7. Notify user of any level-ups
8. Write updated levels to database
```

---

## Lexer Specification

### Overview

The lexer is a state machine that parses a stream of characters (keystrokes) into semantic Vim command tokens. It handles numeric prefixes (e.g., `10j`) and multi-character sequences.

### State Machine

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│    ┌──────────┐         digit (1-9)        ┌────────────┐  │
│    │          │ ─────────────────────────> │            │  │
│    │  None    │                            │ Accum-     │  │
│    │          │ <───────────────────────── │ ulating    │  │
│    └──────────┘      emit token            │ Count      │  │
│         │                                  └────────────┘  │
│         │ command char                           │         │
│         ▼                                        │ digit   │
│    emit Token(1)                                 ▼         │
│                                            continue        │
│                                            accumulating    │
└─────────────────────────────────────────────────────────────┘
```

### States

| State | Description |
|-------|-------------|
| `None` | Default state, ready to process next character |
| `AccumulatingCount(u32)` | Building a numeric prefix |

### Numeric Prefix Rules

1. Leading zeros are treated as separate tokens (e.g., `0j` → `Unhandled("0")`, `MoveVerticalBasic(1)`)
2. Numeric prefixes cap at 999 to prevent overflow
3. Digits not followed by a valid command become `Unhandled`
4. The count multiplier is passed to the resulting token

### Intended Token Mappings

The lexer should recognize and tokenize the following Vim commands:

#### Simple Character Mappings (No Prefix State Required)

| Input | Token | Supports Count |
|-------|-------|----------------|
| `j`, `k` | `MoveVerticalBasic(n)` | Yes |
| `gj`, `gk` | `MoveVerticalBasic(n)` | Yes |
| `h`, `l` | `MoveHorizontalBasic(n)` | Yes |
| `w`, `W`, `e`, `E`, `b`, `B` | `MoveHorizontalChunk(n)` | Yes |
| `x` | `TextManipulationBasic(n)` | Yes |
| `X` | `TextManipulationBasic(n)` | Yes |
| `D` | `DeleteText(n)` | Yes |
| `~` | `TextManipulationAdvanced` | No |
| `u`, `U` | `UndoRedo` | No |
| `.` | `DotRepeat` | No |
| `n`, `N` | `SearchRepeat` | No |
| `;`, `,` | `SearchRepeat` | No |
| `M`, `H`, `L` | `JumpToVertical` | No |
| `p`, `P` | `YankPaste` | No |
| `J` | `TextManipulationBasic(n)` | Yes |
| `G` | `JumpToLineNumber` | With prefix |
| `0` | `Unhandled("0")` | N/A (line start) |

#### Control Character Mappings

| Input | Token |
|-------|-------|
| `<C-U>`, `<C-D>` | `MoveVerticalChunk(n)` |
| `<C-F>`, `<C-B>` | `JumpToVertical` |
| `<C-E>`, `<C-Y>` | `CameraMovement` |
| `<C-R>` | `UndoRedo` |
| `<C-W>` + next | `WindowManagement` |
| `<C-H>`, `<C-J>`, `<C-K>`, `<C-L>` | `WindowManagement` |

#### Multi-Character Sequences

| Input Pattern | Token |
|---------------|-------|
| `gg` | `JumpToLineNumber("")` |
| `[n]gg` | `JumpToLineNumber(n)` |
| `zz`, `zb`, `zt` | `CameraMovement` |
| `f[char]`, `F[char]`, `t[char]`, `T[char]` | `JumpToHorizontal` |
| `dd` | `DeleteText(n)` |
| `yy` | `YankPaste` |
| `cc` | `TextManipulationAdvanced` |
| `d[motion]` | `DeleteText(n)` |
| `y[motion]` | `YankPaste` |
| `c[motion]` | `TextManipulationAdvanced` |
| `r[char]` | `TextManipulationBasic(n)` |
| `R[chars]<Esc>` | `TextManipulationAdvanced` |
| `g~`, `gu`, `gU` + motion | `TextManipulationAdvanced` |
| `m[char]` | `Marks` |
| `'[char]` | `Marks` |
| `` `[char] `` | `Marks` |

#### Command Mode Sequences

| Input Pattern | Token |
|---------------|-------|
| `:[chars]\|enter\|` | `Command(true)` |
| `:[chars]<Esc>` | `Command(false)` |
| `/[chars]\|enter\|` | `CommandSearch(true)` |
| `/[chars]<Esc>` | `CommandSearch(false)` |
| `?[chars]\|enter\|` | `CommandSearch(true)` |
| `?[chars]<Esc>` | `CommandSearch(false)` |
| `:h[elp][chars]\|enter\|` | `HelpPage(true)` |
| `:h[elp][chars]<Esc>` | `HelpPage(false)` |
| `:w\|enter\|` | `SaveFile(true)` |
| `:w<Esc>` | `SaveFile(false)` |
| `:[n]\|enter\|` | `JumpToLineNumber(n)` |

#### Special Sequences

| Input Pattern | Token | Notes |
|---------------|-------|-------|
| `%` (matchit) | `JumpFromContext` | Expands to `:<C-U>call...` |
| `ci)`, `ca)`, etc. | `TextManipulationAdvanced` | Text objects |
| `"[reg][n]p` | `YankPaste` | Register access |

### Pipe-Delimited Special Keys

The lexer handles sanitized special keys from the Lua frontend:

| Sanitized Key | Meaning | Handling |
|---------------|---------|----------|
| `\|enter\|` | Enter/Return | Completes command/search sequences |
| `\|tab\|` | Tab | Adds tab character in command mode |
| `\|backspace\|` | Backspace | Removes character in command mode |
| `\|space\|` | Space | Adds space in command mode |
| `\|escape\|` | Escape | Cancels command/search/replace modes |
| `\|delete\|` | Delete | Currently unhandled |
| `\|up\|` | Arrow Up | Currently unhandled |
| `\|down\|` | Arrow Down | Currently unhandled |
| `\|left\|` | Arrow Left | Currently unhandled |
| `\|right\|` | Arrow Right | Currently unhandled |
| `\|home\|` | Home | Currently unhandled |
| `\|end\|` | End | Currently unhandled |
| `\|pageup\|` | Page Up | Currently unhandled |
| `\|pagedown\|` | Page Down | Currently unhandled |
| `\|insert\|` | Insert | Currently unhandled |

### Lexer Implementation Notes

1. The lexer uses a `Peekable<Chars>` iterator for lookahead
2. When accumulating counts, the full string is preserved for `Unhandled` fallback
3. Multi-character sequences require additional state tracking
4. Command mode (`:`, `/`, `?`) requires accumulating until terminator

---

## Token Definitions

### Token Enum

```rust
pub enum Token {
    MoveVerticalBasic(i32),      // j, k, gj, gk with count
    MoveHorizontalBasic(i32),    // h, l with count
    MoveVerticalChunk(i32),      // <C-U>, <C-D> with count
    MoveHorizontalChunk(i32),    // w, W, e, E, b, B with count
    JumpToHorizontal,            // f, F, t, T + char
    JumpToLineNumber(String),    // gg, G, :[n]
    JumpToVertical,              // M, H, L, <C-F>, <C-B>
    JumpFromContext,             // % (matchit)
    CameraMovement,              // zz, zb, zt, <C-E>, <C-Y>
    WindowManagement,            // <C-W>+key, <C-H/J/K/L>
    TextManipulationBasic(i32),  // x, X, J, r+char with count
    TextManipulationAdvanced,    // c, R, ~, g~/u/U, text objects
    YankPaste,                   // y, p, P, registers
    UndoRedo,                    // u, U, <C-R>
    DotRepeat,                   // .
    CommandSearch(bool),         // /, ? (bool = completed)
    SearchRepeat,                // n, N, ;, , (repeat last search/find)
    DeleteText(i32),             // d+motion, D with count
    Command(bool),               // : commands (bool = completed)
    HelpPage(bool),              // :h, :help (bool = completed)
    SaveFile(bool),              // :w (bool = completed)
    Marks,                       // m{char}, '{char}, `{char}
    Unhandled(String),           // Fallback for unrecognized input
}
```

### Token Notes

- Tokens with `i32` include a count multiplier (default 1)
- Tokens with `bool` indicate completion status (true = executed, false = escaped)
- `Unhandled` captures the raw input string for debugging

---

## Skills System

### Skill Definitions

| Skill | Description | Associated Tokens |
|-------|-------------|-------------------|
| `VerticalNavigation` | Up/down movement | `MoveVerticalBasic`, `MoveVerticalChunk`, `JumpToLineNumber`, `JumpToVertical` |
| `HorizontalNavigation` | Left/right movement | `MoveHorizontalBasic`, `MoveHorizontalChunk`, `JumpToHorizontal` |
| `CodeFlow` | Jumping to locations | `JumpFromContext`, `Marks` |
| `CameraMovement` | Viewport scrolling | `CameraMovement` |
| `WindowManagement` | Window/split operations | `WindowManagement` |
| `TextManipulation` | Text editing | `TextManipulationBasic`, `TextManipulationAdvanced` |
| `Clipboard` | Yank/paste operations | `YankPaste` |
| `Finesse` | Undo/redo/repeat | `UndoRedo`, `DotRepeat` |
| `Search` | Search operations | `CommandSearch` |
| `Knowledge` | Help system usage | `HelpPage` |
| `Saving` | File saving | `SaveFile` |

### Token-to-Skill Mapping with XP

| Token | Skill | Base XP | Notes |
|-------|-------|---------|-------|
| `MoveVerticalBasic(n)` | VerticalNavigation | 1 * n | Scaled by count |
| `MoveHorizontalBasic(n)` | HorizontalNavigation | 1 * n | Scaled by count |
| `MoveVerticalChunk(n)` | VerticalNavigation | 5 * n | Scaled by count |
| `MoveHorizontalChunk(n)` | HorizontalNavigation | 5 * n | Scaled by count |
| `JumpToHorizontal` | HorizontalNavigation | 10 | f/F/t/T jumps |
| `JumpToLineNumber(_)` | VerticalNavigation | 10 | gg, G, :[n] |
| `JumpToVertical` | VerticalNavigation | 10 | M, H, L, C-F, C-B |
| `JumpFromContext` | CodeFlow | 10 | % matchit |
| `CameraMovement` | CameraMovement | 10 | |
| `WindowManagement` | WindowManagement | 10 | |
| `TextManipulationBasic(n)` | TextManipulation | 1 * n | Scaled by count |
| `TextManipulationAdvanced` | TextManipulation | 10 | |
| `YankPaste` | Clipboard | 10 | |
| `UndoRedo` | Clipboard | 10 | |
| `DotRepeat` | Finesse | 10 | |
| `CommandSearch(true)` | Search | 10 | Completed |
| `CommandSearch(false)` | Search | 1 | Escaped |
| `SearchRepeat` | Search | 5 | n/N/;/, repeat |
| `DeleteText(n)` | TextManipulation | 1 * n | Scaled by count |
| `Command(true)` | Finesse | 10 | Generic commands |
| `Command(false)` | Finesse | 1 | Escaped |
| `HelpPage(true)` | Knowledge | 10 | Completed |
| `HelpPage(false)` | Knowledge | 1 | Escaped |
| `SaveFile(true)` | Saving | 10 | Completed |
| `SaveFile(false)` | Saving | 1 | Escaped |
| `Marks` | CodeFlow | 10 | m/'/` marks |
| `Unhandled(_)` | - | 0 | No XP |

---

## Experience & Leveling

### XP Formula

The plugin uses a RuneScape-inspired exponential XP curve:

```
XP for level n = 75 * 1.10409^n
```

Cumulative XP is precomputed for levels 1-99.

### Level Thresholds

| Level | Cumulative XP |
|-------|---------------|
| 1 | 0 |
| 2 | 83 |
| 5 | 388 |
| 10 | 1,154 |
| 20 | 4,470 |
| 30 | 13,363 |
| 40 | 37,224 |
| 50 | 101,333 |
| 60 | 273,742 |
| 70 | 737,627 |
| 80 | 1,986,068 |
| 90 | 5,346,332 |
| 99 | 13,034,431 |

### Level Calculation

```rust
fn get_level_for_exp(exp: i32) -> i32 {
    // Binary search through cumulative XP table
    // Returns highest level where cumulative_xp[level] <= exp
    // Capped at 99
}
```

### Level-Up Notifications

When a skill levels up, the plugin sends a Neovim notification:

```
[Vimscape2007] VerticalNavigation leveled up! (Level 5)
```

---

## Database Schema

### SQLite Database

**Table: `skills`**

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | INTEGER | PRIMARY KEY | Auto-increment ID |
| `name` | TEXT | UNIQUE, NOT NULL | Skill name |
| `exp` | INTEGER | DEFAULT 0 | Total experience points |
| `level` | INTEGER | DEFAULT 1 | Current level (1-99) |

### Database Operations

| Operation | Function | Description |
|-----------|----------|-------------|
| Initialize | `create_tables()` | Create schema if not exists |
| Seed | `populate_skills_enum_table()` | Insert all 11 skills |
| Read | `get_skill_data()` | Fetch all skills |
| Read | `get_skill_details_from_db()` | Fetch single skill |
| Write | `write_exp_to_table()` | Increment XP for skill |
| Write | `write_levels_to_table()` | Update levels |

---

## User Interface

### Stats Window

A centered floating window displaying all skills in a multi-column layout:

```
╔══════════════════════════════════════════════════════════════╗
║                          Skills                               ║
╠══════════════════════════════════════════════════════════════╣
║ ┌───────────────────────┐ ┌───────────────────────┐          ║
║ │ VerticalNavigation    │ │ HorizontalNavigation  │          ║
║ │ Level: 12             │ │ Level: 8              │          ║
║ │ XP: 4,521             │ │ XP: 1,203             │          ║
║ └───────────────────────┘ └───────────────────────┘          ║
║ ┌───────────────────────┐ ┌───────────────────────┐          ║
║ │ CodeFlow              │ │ TextManipulation      │          ║
║ │ Level: 5              │ │ Level: 7              │          ║
║ │ XP: 388               │ │ XP: 892               │          ║
║ └───────────────────────┘ └───────────────────────┘          ║
║ ...                                                           ║
╠══════════════════════════════════════════════════════════════╣
║                    [q]uit | [d]etails                         ║
╚══════════════════════════════════════════════════════════════╝
```

### Details Popup

A cursor-relative popup showing detailed skill information:

```
╭────────────────────────╮
│ Experience - 4,521     │
│ Level - 12             │
╰────────────────────────╯
```

### Window Configuration

| Window | Size | Position | Border |
|--------|------|----------|--------|
| Stats | 50% editor | Centered | Double |
| Details | 24x4 | Cursor-relative | Rounded |

---

## Configuration

### Default Configuration

```lua
{
    db_path = plugin_directory .. "/",  -- Plugin's own directory
    db_name = "vimscape.db",            -- Database filename
    batch_size = 1000,
    log_level = vim.log.levels.INFO,    -- Integer log level
    batch_notify = false
}
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `db_path` | string | Plugin directory | Directory for SQLite database |
| `db_name` | string | `"vimscape.db"` | Database filename |
| `batch_size` | number | 1000 | Keystrokes before processing batch |
| `log_level` | integer | `vim.log.levels.INFO` | Minimum log level for notifications |
| `batch_notify` | boolean | false | Show notification after each batch |

**Note:** The full database path is constructed as `db_path .. db_name`.

### Setup Example

```lua
require('vimscape2007').setup({
    batch_size = 100,
    batch_notify = true
})
```

---

## Build & Development

### Build Process

```bash
cd vimscape_backend
./build-dev.sh
```

This compiles the Rust backend and copies `libvimscape_backend.so` to `lua/vimscape_backend.so`.

### Dependencies

**Rust:**
- `nvim-oxi 0.6.0` - Neovim API bindings
- `rusqlite 0.32.1` - SQLite bindings
- `once_cell 1.19` - Lazy static initialization
- `serde 1.0.216` - Serialization

### Platform Notes

- macOS requires special rustflags for dynamic linking (`-undefined dynamic_lookup`)
- Configured in `.cargo/config.toml` for both x86_64 and aarch64

---

## Future Considerations

1. **Arrow Key Support**: Handle `|up|`, `|down|`, `|left|`, `|right|` in the lexer
2. **Navigation Key Support**: Handle `|home|`, `|end|`, `|pageup|`, `|pagedown|` in the lexer
3. **Visual Mode**: Currently not tracked
4. **Macro Recording**: Could track `q` register usage
5. **Register Access**: Full support for `"[reg]` prefix patterns

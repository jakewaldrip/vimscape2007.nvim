# AGENTS.md - Vimscape2007.nvim

This document provides guidelines for AI coding agents working in this repository.

## Project Overview

Vimscape2007.nvim is a hybrid **Lua/Rust Neovim plugin** that gamifies Vim usage in the style of RuneScape (2007 era). It tracks keystrokes and awards experience points to various skills.

- **Primary Languages**: Lua (Neovim plugin frontend) + Rust (backend/engine)
- **Framework**: nvim-oxi (Rust bindings for Neovim)
- **Target**: Neovim 0.11+
- **Database**: SQLite (via rusqlite)

## Directory Structure

```
vimscape2007.nvim/
├── lua/
│   ├── vimscape2007/
│   │   └── init.lua         # Plugin entry point, commands, UI
│   ├── config.lua           # Configuration defaults
│   ├── globals.lua          # Runtime state management
│   ├── keys.lua             # Keystroke capture/sanitization
│   ├── utils.lua            # Utility functions
│   └── window_config.lua    # Floating window configurations
├── vimscape_backend/
│   ├── Cargo.toml
│   ├── build-dev.sh         # Build script
│   └── src/
│       ├── lib.rs           # Plugin entry, nvim-oxi exports
│       ├── api.rs           # Public API for Lua
│       ├── lexer.rs         # Vim command lexer (state machine)
│       ├── token.rs         # Token enum definitions
│       ├── parse_utils.rs   # Token-to-skill mapping
│       ├── skills.rs        # Skills enum
│       ├── levels.rs        # XP curve/level calculations
│       ├── skill_data.rs    # Display formatting
│       └── db.rs            # SQLite persistence
├── notes/                   # Planning docs, research, tickets
└── SPEC.md                  # Comprehensive technical specification
```

## Build Commands

### Rust Backend

```bash
# Build and copy shared library to lua/ directory
cd vimscape_backend && ./build-dev.sh

# Or manually:
cd vimscape_backend && cargo build && cp target/debug/libvimscape_backend.so ../lua/vimscape_backend.so
```

### Linting

```bash
# Rust - clippy (configured with pedantic)
cd vimscape_backend && cargo clippy
```

## Testing

### Rust Tests

```bash
# Run all tests
cd vimscape_backend && cargo test

# Run a single test by name
cd vimscape_backend && cargo test test_name_here

# Run tests in a specific module
cd vimscape_backend && cargo test lexer::tests::test_numeric_prefix_basic

# Examples:
cargo test test_delete_line
cargo test test_search_completed
```

**Note**: There are no Lua tests in this codebase currently.

**Important**: Do NOT run all tests unless explicitly requested. Run specific tests when verifying changes.

## Code Style Guidelines

### Rust

#### Clippy Configuration

The project uses strict linting. See `lib.rs`:
```rust
#![warn(clippy::pedantic)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
```

#### Imports

Group imports in this order with blank lines between groups:
1. Standard library (`std::`)
2. External crates (`rusqlite::`, `nvim_oxi::`)
3. Internal modules (`crate::`)

```rust
use std::{collections::HashMap, path::Path};

use rusqlite::Connection;

use crate::{
    db::{create_tables, get_skill_data},
    levels::get_levels_diff,
};
```

#### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Functions | `snake_case` | `process_batch`, `get_skill_data` |
| Types/Enums | `PascalCase` | `Token`, `Skills`, `SkillData` |
| Constants | `SCREAMING_SNAKE_CASE` | `XP_BASE`, `MAX_NUM_COLS` |
| Enum variants | `PascalCase` | `MoveVerticalBasic`, `JumpToHorizontal` |

#### Error Handling

- Use `Result` and `Option` types appropriately
- Prefer `if let Some(x) = ...` and `if let Ok(x) = ...` patterns
- Use early returns: `let Ok(conn) = Connection::open(...) else { return ...; }`
- Use `expect()` for critical failures with descriptive messages
- Use `println!` for non-critical error logging

#### Documentation

- Module-level doc comments (`//!`) for complex modules
- Function doc comments (`///`) for public API functions
- Inline comments for complex logic

#### Tests

- Place tests in the same file using `#[cfg(test)]` module
- Use descriptive test names: `test_numeric_prefix_basic`
- Use `assert!(matches!(...))` for enum matching

### Lua

#### Module Pattern

Always use the standard module pattern:
```lua
---@class ClassName
---@field field_name type Description
local M = {}

M.function_name = function(args)
    -- implementation
end

return M
```

#### Imports

Place requires at the top of the file:
```lua
local keys = require("keys")
local utils = require("utils")
local globals = require("globals")
```

#### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Variables | `snake_case` | `typed_letters`, `db_path` |
| Functions | `snake_case` | `sanitize_key`, `record_keys` |
| Module tables | `M` | Standard convention |

#### Type Annotations

Use LuaCATS/EmmyLua annotations for type safety:
```lua
---@class Config
---@field db_path string [required] Path to database
---@field batch_size integer Number of keys before processing
```

#### Error Handling

- Use early returns for invalid states
- Use `vim.notify()` for user-facing messages
- Do not throw errors; return nil or false for failure cases

#### Formatting

- Use tabs for indentation
- Opening brace on same line as statement
- Prefer double quotes for strings

## Key Architecture Patterns

### FFI Bridge (Lua <-> Rust)

Rust functions are exposed via `nvim-oxi` Dictionary:
```rust
// Rust side (lib.rs)
Dictionary::from_iter([
    ("process_batch", Object::from(process_batch_fn)),
    ("get_user_data", Object::from(get_user_data_fn)),
])
```

```lua
-- Lua side
local vimscape = require("vimscape_backend")
vimscape.process_batch(string_value, db_path)
```

### State Machine (Lexer)

The lexer uses a state machine with states:
- `None` - Default, ready for next character
- `AccumulatingCount` - Building numeric prefix
- `OperatorPending` - Waiting for motion after operator
- `CommandMode` - Accumulating `:` command
- `SearchMode` - Accumulating `/` or `?` search
- `ReplaceMode` - In replace mode
- `CaseOperatorPending` - After `g~`, `gu`, `gU`

### Database

- SQLite via rusqlite
- Single `skills` table with: id, name, exp, level
- Database file: `teste.db` in configured path

## Important Files to Reference

- `SPEC.md` - Comprehensive technical specification with token definitions, skill mappings, XP values
- `vimscape_backend/src/lexer.rs` - Core lexer implementation with extensive tests
- `vimscape_backend/src/token.rs` - Token enum definitions
- `lua/vimscape2007/init.lua` - Plugin entry point and user commands

## Platform Notes

- macOS requires special rustflags for dynamic linking
- Configured in `.cargo/config.toml` for both x86_64 and aarch64:
  ```toml
  rustflags = ["-C", "link-args=-undefined dynamic_lookup"]
  ```

## User Commands

| Command | Description |
|---------|-------------|
| `:Vimscape stats` | Open skills display window |
| `:Vimscape details` | Show details for skill under cursor |
| `:Vimscape toggle` | Toggle recording on/off |

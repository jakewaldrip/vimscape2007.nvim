# vimscape2007.nvim

Neovim plugin to gamify development in the spirit of old school RuneScape.

Track your Vim usage and earn XP across 11 skills, leveling from 1-99 on a RuneScape-inspired XP curve.

### Skills

| Skill | Description | Example Commands |
|-------|-------------|------------------|
| **VerticalNavigation** | Up/down movement | `j`, `k`, `gj`, `gk`, `Ctrl-U`, `Ctrl-D`, `gg`, `G` |
| **HorizontalNavigation** | Left/right movement | `h`, `l`, `w`, `W`, `e`, `E`, `b`, `B`, `f`, `F`, `t`, `T` |
| **CodeFlow** | Jumping to locations | `%`, marks (`m`, `'`, `` ` ``) |
| **CameraMovement** | Viewport scrolling | `zz`, `zt`, `zb`, `Ctrl-E`, `Ctrl-Y` |
| **WindowManagement** | Window/split operations | `Ctrl-W` commands |
| **TextManipulation** | Text editing | `x`, `d`, `c`, `r`, `~`, `J` |
| **Clipboard** | Yank/paste operations | `y`, `p`, `P`, `u`, `Ctrl-R` |
| **Finesse** | Undo/redo/repeat | `.` (dot repeat), `:` commands |
| **Search** | Search operations | `/`, `?`, `n`, `N`, `;`, `,` |
| **Knowledge** | Help system usage | `:help`, `:h` |
| **Saving** | File saving | `:w` |

<!-- TODO: Add screenshots and/or GIF demo of the plugin in action -->

## Requirements

- Neovim 0.11+
- Rust toolchain (only if building from source)

## Installation

### lazy.nvim

```lua
-- Pre-built binaries (recommended)
{
    "jakewaldrip/vimscape2007.nvim",
    version = "*",
    opts = {},
}

-- Build from source
{
    "jakewaldrip/vimscape2007.nvim",
    opts = {},
}
```

### Manual

1. Clone the repository
2. Build the Rust backend:
   ```bash
   cd vimscape_backend && cargo build --release
   ```
3. Copy the compiled library to `lua/`:
   ```bash
   # macOS
   cp vimscape_backend/target/release/libvimscape_backend.dylib lua/vimscape_backend.so
   # Linux
   cp vimscape_backend/target/release/libvimscape_backend.so lua/vimscape_backend.so
   ```
4. Add the plugin to your Neovim configuration

## Configuration

```lua
{
    "jakewaldrip/vimscape2007.nvim",
    version = "*",
    ---@type Config
    opts = {
        -- Directory path where the SQLite database will be stored
        db_path = vim.fn.stdpath("data") .. "/vimscape2007/",

        -- Filename for the database file
        db_name = "vimscape.db",

        -- Number of keystrokes buffered before processing a batch
        batch_size = 1000,

        -- Minimum log level for notifications (vim.log.levels)
        log_level = vim.log.levels.INFO,

        -- Enable token logging to file (for integration testing/debugging)
        token_log = false,

        -- Map of physical keys to substituted keys for the lexer
        -- Useful if you remap keys at the OS/keyboard level
        -- Example: { [";"] = ":" }
        key_overrides = {},

        -- Enable recording when the plugin starts (default: true)
        recording_on = true,
    },
}
```

## Commands

| Command | Description |
|---------|-------------|
| `:Vimscape stats` | Open skills display window |
| `:Vimscape details` | Show details for skill under cursor |
| `:Vimscape toggle` | Toggle keystroke recording on/off |
| `:Vimscape flush` | Manually process and save buffered keystrokes |

In the stats window, press `q` to close or `d` to show details for the skill under cursor.

## Tips & Troubleshooting

- **Database location** -- The database is stored in Neovim's data directory (`vim.fn.stdpath("data")`) by default, which persists safely across plugin updates. You can customize `db_path` if you prefer a different location.

- **Leader key** -- Only `<Space>` is supported as a leader key. If you use a different leader (e.g., `,` or `\`), echoed leader bindings will be misinterpreted by the lexer, causing inaccurate XP tracking.

- **Key remaps** -- The lexer sees physical keys, not remapped ones. If you remap keys at the Vim level (e.g., `;` to `:`), add them to `key_overrides` so the lexer interprets them correctly: `key_overrides = { [";"] = ":" }`.

- **XP lost on exit** -- Keystrokes are buffered in memory until `batch_size` is reached. There is no automatic flush when Neovim exits, so any buffered keystrokes are lost. Use `:Vimscape flush` before quitting, or lower `batch_size` to reduce the risk.

- **Tracked modes** -- Only normal mode keystrokes earn XP. Insert mode is skipped entirely. Visual mode and macro commands are captured but don't earn XP yet.

- **Untracked motions** -- Some normal mode commands don't earn XP yet, including `0`, `$`, `^`, arrow keys, visual mode operators, macros (`q`/`@`), and register prefixes (`"`). These are planned for future releases.

## License

[MIT](LICENSE.md)

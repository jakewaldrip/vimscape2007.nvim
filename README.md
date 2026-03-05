# Warning

Though this plugin is very close to being released, its not _quite_ ready. Check back soon for an official release

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

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `db_path` | string | Plugin directory | Directory for the SQLite database |
| `db_name` | string | `"vimscape.db"` | Database filename |
| `batch_size` | number | `1000` | Keystrokes before processing batch |
| `log_level` | integer | `vim.log.levels.INFO` | Minimum log level for notifications |
| `batch_notify` | boolean | `false` | Show notification after each batch |

## Commands

| Command | Description |
|---------|-------------|
| `:Vimscape stats` | Open skills display window |
| `:Vimscape details` | Show details for skill under cursor |
| `:Vimscape toggle` | Toggle keystroke recording on/off |

In the stats window, press `q` to close or `d` to show details for the skill under cursor.

## License

[MIT](LICENSE)

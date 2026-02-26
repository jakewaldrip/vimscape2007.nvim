# vimscape2007.nvim

Neovim plugin to gamify development in the spirit of old school RuneScape.

## Description

Vimscape2007 transforms your Vim experience into an RPG-style progression system. As you use Vim commands, you earn experience points across 11 distinct skills, leveling from 1-99 using a RuneScape-inspired XP curve.

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

## Screenshots / Demo

<!-- TODO: Add screenshots and/or GIF demo of the plugin in action -->

## Requirements

- Neovim 0.11+
- Rust toolchain (for building the backend)

## Installation

### Using lazy.nvim

```lua
{
    "jakewaldrip/vimscape2007.nvim",
    build = "cd vimscape_backend && ./build-dev.sh",
    config = function()
        require("vimscape2007").setup()
    end,
}
```

### Using packer.nvim

```lua
use {
    "jakewaldrip/vimscape2007.nvim",
    run = "cd vimscape_backend && ./build-dev.sh",
    config = function()
        require("vimscape2007").setup()
    end,
}
```

### Manual Installation

1. Clone the repository
2. Build the Rust backend:
   ```bash
   cd vimscape_backend && ./build-dev.sh
   ```
3. Add the plugin to your Neovim configuration

## Configuration

```lua
require("vimscape2007").setup({
    -- Directory for the database file (default: plugin directory)
    db_path = vim.fn.stdpath("data") .. "/vimscape/",

    -- Database filename (default: "vimscape.db")
    db_name = "vimscape.db",

    -- Number of keystrokes before processing (default: 1000)
    batch_size = 1000,

    -- Minimum log level for notifications (default: vim.log.levels.INFO)
    log_level = vim.log.levels.INFO,

    -- Show notification after each batch processing (default: false)
    batch_notify = false,
})
```

### Configuration Options

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

### Stats Window Keymaps

| Key | Action |
|-----|--------|
| `q` | Close window |
| `d` | Show details for skill under cursor |

## License

[MIT](LICENSE)

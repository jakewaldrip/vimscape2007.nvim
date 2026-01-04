# Vimscape2007 - Neovim Plugin Specification

## Project Overview
Vimscape2007 is a neovim plugin that gamifies vim usage by tracking user movements and commands, converting them into experience points across twelve skill categories. The plugin features a Rust backend that processes vim commands through a lexical analyzer, maps them to skills, maintains persistent levels in SQLite, and provides responsive UI display.

## Architecture

### High-Level Components
```
Neovim (lua) → Rust Backend (nvim-oxi) → Lexer → Token → Skill Mapping → XP Calculation → SQLite Database → UI Display
```

### Entry Points (lib.rs:17-29)
- `process_batch(cmd_string: String)` - Process vim command sequences
- `get_user_data(width: i32)` - Retrieve formatted skill display
- `setup_tables()` - Initialize database schema  
- `get_skill_details(skill_name: String)` - Get individual skill info

## Current Implementation

### 1. Command Processing Pipeline (api.rs:16-45)
1. **Lexical Analysis** - `Lexer::new()` tokenizes input string
2. **Token Iteration** - Loop processes each token with numeric prefix
3. **Skill Mapping** - `parse_action_into_skill()` converts tokens to XP
4. **Database Updates** - Write XP and level changes to SQLite
5. **User Notifications** - Display level-up achievements

### 2. Lexer Implementation (lexer.rs:11-120)

#### State Machine Pattern
- `State::Normal` - Ready for new command
- `State::Accumulating` - Building numeric prefix
- `State::Finished` - End of input

#### Current Supported Commands
- **Vertical Movement**: `j`, `k` with numeric prefixes (e.g., `3j`)
- **Horizontal Movement**: `w`, `b` with numeric prefixes (e.g., `5w`)
- **Numeric Handling**: Counts capped at 999, resets on command completion

### 3. Token System (token.rs:1-83)

#### Movement Tokens
- `MoveVerticalBasic` - j/k movements
- `MoveHorizontalBasic` - w/b movements  
- `MoveVerticalChunk` - {/} paragraph movements (planned)
- `MoveHorizontalChunk` - 0/^/$ line movements (planned)

#### Navigation Tokens (Planned)
- `JumpToHorizontal` - f/F/t/T character jumps
- `JumpToLineNumber` - :line_number jumps
- `JumpToVertical` - G/gg file positioning
- `JumpFromContext` - %/{/[/(/ matching

#### Editing Tokens (Planned)
- `TextManipulationBasic` - i/a/o/O insert modes
- `TextManipulationAdvanced` - cw/c$/cc change commands
- `DeleteText` - d/x delete operations

#### System Tokens (Planned)
- `CameraMovement` - zz/zt/zb view positioning
- `WindowManagement` - <C-w> window commands
- `YankPaste` - y/p/put operations
- `UndoRedo` - u/<C-r> operations
- `DotRepeat` - . command repetition

#### Command Tokens (Planned)
- `CommandSearch` - /?/?n/N searching
- `Command` - : command mode operations
- `HelpPage` - :help navigation
- `SaveFile` - :w/:wq saving

### 4. Skill Categories (skills.rs:1-67)

#### Navigation Skills
- `VerticalNavigation` - j/k movements
- `HorizontalNavigation` - w/b movements
- `CodeFlow` - Jump-based navigation

#### Interface Skills  
- `CameraMovement` - View positioning
- `WindowManagement` - Window operations

#### Editing Skills
- `TextManipulation` - Insert/change/delete operations
- `Clipboard` - Yank/paste operations

#### Advanced Skills
- `Finesse` - Precision movements and commands
- `Search` - Search and navigation operations
- `Knowledge` - Help and command usage
- `Saving` - File operations

### 5. Experience System (parse_utils.rs:3-87)

#### Current XP Values
- **Basic Movements**: 1 XP per unit (j, k, w, b)
- **Chunk Movements**: 5 XP per unit (planned: {, }, 0, ^, $)
- **Advanced Operations**: 10 XP each (editing, system commands)
- **Command Escaping**: 1 XP for `<Esc>` escaped commands vs 10 XP normal

#### XP Accumulation
- Numeric prefixes multiply XP (e.g., `3j` = 3 XP)
- Experience tracked per skill type in HashMap
- Batch processing accumulates before database write

### 6. Level System (levels.rs:10-54)

#### Progressive Formula
- Base: `XP_BASE = 75.0`
- Multiplier: `XP_MULTIPLIER = 1.10409`
- Formula: `XP_BASE * XP_MULTIPLIER^level`
- Range: Levels 1-99
- Pre-calculated cumulative XP table for efficiency

#### Level Calculation
- Binary search on cumulative XP array
- Level-ups trigger neovim notifications
- Level display shows current level and progress

### 7. Database Layer (db.rs:43-62)

#### Schema
```sql
CREATE TABLE skills (
  id integer primary key,
  name text not null unique,
  exp integer not null default 0,
  level integer not null default 1
);
```

#### Operations
- `create_tables()` - Schema initialization with 12 skill rows
- `write_exp_to_table()` - Increment experience values
- `write_levels_to_table()` - Update skill levels
- `get_skill_data()` - Retrieve all skill information

### 8. Display System (skill_data.rs:20-92)

#### Responsive Layout
- Column count: 1-3 based on screen width
- Column width: Fixed 25 characters
- Unicode box drawing for borders
- Centered skill names and levels

#### Display Features
- Multi-column responsive design
- Skill grouping and ordering
- Level progress indication
- Error handling for small screens

## Planned Features

### Enhanced Lexer Support
- Complete vim command grammar
- Modal state tracking (normal, insert, visual)
- Command chaining and composition
- Register and mark operations

### Advanced Skill Tracking
- Per-file skill statistics
- Command efficiency metrics
- Time-based skill development
- Streak and combo systems

### Extended Database Schema
- Command history logging
- Performance metrics storage
- User preference tracking
- Achievement system data

### UI Enhancements
- Real-time skill notifications
- Interactive skill tree visualization
- Progress charts and analytics
- Customizable display themes

## Technical Implementation Details

### Dependencies
- **nvim-oxi 0.6.0** - Neovim plugin framework
- **rusqlite 0.32.1** - SQLite database operations
- **serde 1.0.216** - Serialization (future use)
- **once_cell 1.19** - Static initialization

### Error Handling
- Database errors with graceful fallbacks
- Lexing errors captured as Unhandled tokens
- Experience overflow prevention through count capping
- Display errors with minimal screen handling

### Performance Considerations
- Pre-calculated level XP table
- Batch database operations
- Efficient lexer state machine
- Responsive display scaling

### Configuration Constants
- Database file: "teste.db"
- Maximum command count: 999
- Column width: 25 characters
- Level cap: 99
- XP multiplier: 1.10409

## Development Patterns

### Design Patterns Used
- **State Machine** - Lexer command parsing
- **Visitor Pattern** - Token-to-skill mapping
- **Repository Pattern** - Database access layer
- **Factory Pattern** - Token creation
- **Observer Pattern** - Level-up notifications

### Code Organization
- `lib.rs` - Plugin entry points
- `api.rs` - Core processing logic
- `lexer.rs` - Command tokenization
- `token.rs` - Token type definitions
- `skills.rs` - Skill category mappings
- `parse_utils.rs` - XP calculation logic
- `levels.rs` - Level progression system
- `db.rs` - Database operations
- `skill_data.rs` - UI formatting

## Future Architecture Considerations

### Extensibility Points
- Token enum expansion for new commands
- Skill category addition framework
- XP value tuning system
- Display theme customization

### Scalability
- Database schema migration strategy
- Performance optimization for large histories
- Memory management for extended tracking
- Plugin configuration management

This specification serves as the authoritative reference for Vimscape2007 development, encompassing both the current movement tracking implementation and the comprehensive vision for a complete vim gamification system.
# Vimscape2007 Lexer Implementation Plan

This document outlines a multi-phase plan to implement all unhandled token types in the lexer. Each phase is self-contained and can be executed independently by an agent using only this plan and `SPEC.md`.

---

## Current State Analysis

### Implemented
- `MoveVerticalBasic(i32)` - `j`, `k` with numeric prefix
- `Unhandled(String)` - fallback for unrecognized input
- Numeric prefix accumulation (capped at 999)

### Bug to Fix
- `w`, `b` are incorrectly mapped to `MoveHorizontalBasic` - should be `MoveHorizontalChunk` per spec

---

## Phase 1: Simple Single-Character Tokens

**Goal:** Implement all simple single-character commands that don't require lookahead or state tracking.

**File:** `vimscape_backend/src/lexer.rs`

### Tasks

- [x] 1. **Fix existing bug:** Change `w`, `b` to emit `MoveHorizontalChunk` instead of `MoveHorizontalBasic`

- [x] 2. **Add `h`, `l` → `MoveHorizontalBasic(n)`**
   - Supports numeric prefix
   - Example: `h` → `MoveHorizontalBasic(1)`, `10l` → `MoveHorizontalBasic(10)`

- [x] 3. **Add `W`, `e`, `E`, `B` → `MoveHorizontalChunk(n)`**
   - Supports numeric prefix
   - Add to existing `w`, `b` match arm

- [x] 4. **Add `u`, `U` → `UndoRedo`**
   - No numeric prefix support
   - Simple single character match

- [x] 5. **Add `.` → `DotRepeat`**
   - No numeric prefix support
   - Simple single character match

- [x] 6. **Add `M`, `H`, `L` → `JumpToVertical`**
   - No numeric prefix support
   - Simple single character match

- [x] 7. **Add `p`, `P` → `YankPaste`**
   - No numeric prefix support (register handling is separate phase)
   - Simple single character match

- [x] 8. **Add `x` → `TextManipulationBasic(n)`**
   - Supports numeric prefix
   - Example: `x` → `TextManipulationBasic(1)`, `5x` → `TextManipulationBasic(5)`

- [x] 9. **Add `J` → `TextManipulationBasic(n)`**
   - Supports numeric prefix
   - Example: `J` → `TextManipulationBasic(1)`, `3J` → `TextManipulationBasic(3)`

- [x] 10. **Add `G` → `JumpToLineNumber(String)`**
    - With numeric prefix: `10G` → `JumpToLineNumber("10")`
    - Without prefix: `G` → `JumpToLineNumber("")` (end of file)

### Test Cases to Add

```rust
#[test]
fn test_horizontal_basic() {
    let mut lexer = Lexer::new("h5l");
    assert!(matches!(lexer.next_token(), Some(Token::MoveHorizontalBasic(1))));
    assert!(matches!(lexer.next_token(), Some(Token::MoveHorizontalBasic(5))));
}

#[test]
fn test_horizontal_chunk() {
    let mut lexer = Lexer::new("wWeEbB");
    for _ in 0..6 {
        assert!(matches!(lexer.next_token(), Some(Token::MoveHorizontalChunk(1))));
    }
}

#[test]
fn test_undo_redo() {
    let mut lexer = Lexer::new("uU");
    assert!(matches!(lexer.next_token(), Some(Token::UndoRedo)));
    assert!(matches!(lexer.next_token(), Some(Token::UndoRedo)));
}

#[test]
fn test_dot_repeat() {
    let mut lexer = Lexer::new(".");
    assert!(matches!(lexer.next_token(), Some(Token::DotRepeat)));
}

#[test]
fn test_jump_to_vertical() {
    let mut lexer = Lexer::new("MHL");
    for _ in 0..3 {
        assert!(matches!(lexer.next_token(), Some(Token::JumpToVertical)));
    }
}

#[test]
fn test_yank_paste() {
    let mut lexer = Lexer::new("pP");
    assert!(matches!(lexer.next_token(), Some(Token::YankPaste)));
    assert!(matches!(lexer.next_token(), Some(Token::YankPaste)));
}

#[test]
fn test_text_manipulation_basic_x() {
    let mut lexer = Lexer::new("x5x");
    assert!(matches!(lexer.next_token(), Some(Token::TextManipulationBasic(1))));
    assert!(matches!(lexer.next_token(), Some(Token::TextManipulationBasic(5))));
}

#[test]
fn test_text_manipulation_basic_j() {
    let mut lexer = Lexer::new("J3J");
    assert!(matches!(lexer.next_token(), Some(Token::TextManipulationBasic(1))));
    assert!(matches!(lexer.next_token(), Some(Token::TextManipulationBasic(3))));
}

#[test]
fn test_jump_to_line_g() {
    let mut lexer = Lexer::new("G10G");
    assert!(matches!(lexer.next_token(), Some(Token::JumpToLineNumber(ref s)) if s == ""));
    assert!(matches!(lexer.next_token(), Some(Token::JumpToLineNumber(ref s)) if s == "10"));
}
```

### Acceptance Criteria
- All tests pass
- No regressions in existing tests
- `cargo test` succeeds

---

## Phase 2: Control Character Tokens

**Goal:** Implement control character sequences (`<C-X>` format).

**File:** `vimscape_backend/src/lexer.rs`

### Prerequisites
- Phase 1 complete

### Background
Control characters arrive from the Lua frontend in the format `<C-X>` where X is the character. The lexer must recognize the `<` character and parse the full control sequence.

### Tasks

- [x] 1. **Add helper function to parse control sequences:**
   ```rust
   fn try_parse_control_sequence(&mut self) -> Option<String> {
       // Peek ahead to check for <C-X> pattern
       // Return Some("C-X") if valid, None otherwise
       // Must handle: <C-U>, <C-D>, <C-F>, <C-B>, <C-E>, <C-Y>, <C-R>, <C-W>, <C-H>, <C-J>, <C-K>, <C-L>
   }
   ```

- [x] 2. **Add `<C-U>`, `<C-D>` → `MoveVerticalChunk(n)`**
   - Supports numeric prefix
   - Example: `<C-U>` → `MoveVerticalChunk(1)`, `5<C-D>` → `MoveVerticalChunk(5)`

- [x] 3. **Add `<C-F>`, `<C-B>` → `JumpToVertical`**
   - No numeric prefix support

- [x] 4. **Add `<C-E>`, `<C-Y>` → `CameraMovement`**
   - No numeric prefix support

- [x] 5. **Add `<C-R>` → `UndoRedo`**
   - No numeric prefix support

- [x] 6. **Add `<C-H>`, `<C-J>`, `<C-K>`, `<C-L>` → `WindowManagement`**
   - No numeric prefix support

- [x] 7. **Add `<C-W>` → `WindowManagement`**
   - Consumes the next character as well (e.g., `<C-W>s`, `<C-W>v`)
   - The follow-up character is part of the command but not parsed separately

### State Machine Update

When encountering `<`:
1. Attempt to parse control sequence
2. If successful, emit appropriate token
3. If not a valid control sequence, emit `Unhandled("<")`

### Test Cases to Add

```rust
#[test]
fn test_control_vertical_chunk() {
    let mut lexer = Lexer::new("<C-U><C-D>5<C-U>");
    assert!(matches!(lexer.next_token(), Some(Token::MoveVerticalChunk(1))));
    assert!(matches!(lexer.next_token(), Some(Token::MoveVerticalChunk(1))));
    assert!(matches!(lexer.next_token(), Some(Token::MoveVerticalChunk(5))));
}

#[test]
fn test_control_jump_vertical() {
    let mut lexer = Lexer::new("<C-F><C-B>");
    assert!(matches!(lexer.next_token(), Some(Token::JumpToVertical)));
    assert!(matches!(lexer.next_token(), Some(Token::JumpToVertical)));
}

#[test]
fn test_control_camera() {
    let mut lexer = Lexer::new("<C-E><C-Y>");
    assert!(matches!(lexer.next_token(), Some(Token::CameraMovement)));
    assert!(matches!(lexer.next_token(), Some(Token::CameraMovement)));
}

#[test]
fn test_control_undo_redo() {
    let mut lexer = Lexer::new("<C-R>");
    assert!(matches!(lexer.next_token(), Some(Token::UndoRedo)));
}

#[test]
fn test_control_window_management() {
    let mut lexer = Lexer::new("<C-W>s<C-W>v<C-H><C-J><C-K><C-L>");
    for _ in 0..6 {
        assert!(matches!(lexer.next_token(), Some(Token::WindowManagement)));
    }
}
```

### Acceptance Criteria
- All new tests pass
- No regressions
- `cargo test` succeeds

---

## Phase 3: Two-Character Sequences (g-prefix and z-prefix)

**Goal:** Implement `g` and `z` prefixed commands that require one character of lookahead.

**File:** `vimscape_backend/src/lexer.rs`

### Prerequisites
- Phase 1 complete

### Tasks

- [x] 1. **Add `g` prefix handling:**
   - `gj`, `gk` → `MoveVerticalBasic(n)` (supports numeric prefix)
   - `gg` → `JumpToLineNumber(String)` (with optional numeric prefix: `10gg` → `JumpToLineNumber("10")`)
   - `gJ` → `TextManipulationBasic(n)` (supports numeric prefix)
   - `g~`, `gu`, `gU` → requires motion (see Phase 5)
   - Unrecognized `g` + char → `Unhandled("gX")`

- [x] 2. **Add `z` prefix handling:**
   - `zz` → `CameraMovement`
   - `zb` → `CameraMovement`
   - `zt` → `CameraMovement`
   - Unrecognized `z` + char → `Unhandled("zX")`

### Implementation Notes

When `g` is encountered:
1. Check if there's a numeric prefix accumulated
2. Peek at next character
3. Match on the pair and emit appropriate token
4. For `gg` with prefix: emit `JumpToLineNumber(prefix_string)`

When `z` is encountered:
1. Peek at next character
2. Match on the pair and emit `CameraMovement`
3. Note: `zz` in Vim actually renders as `zzz` per spec comments

### Test Cases to Add

```rust
#[test]
fn test_g_vertical_movement() {
    let mut lexer = Lexer::new("gj5gk");
    assert!(matches!(lexer.next_token(), Some(Token::MoveVerticalBasic(1))));
    assert!(matches!(lexer.next_token(), Some(Token::MoveVerticalBasic(5))));
}

#[test]
fn test_gg_jump() {
    let mut lexer = Lexer::new("gg10gg");
    assert!(matches!(lexer.next_token(), Some(Token::JumpToLineNumber(ref s)) if s == ""));
    assert!(matches!(lexer.next_token(), Some(Token::JumpToLineNumber(ref s)) if s == "10"));
}

#[test]
fn test_gj_text_manipulation() {
    let mut lexer = Lexer::new("gJ3gJ");
    assert!(matches!(lexer.next_token(), Some(Token::TextManipulationBasic(1))));
    assert!(matches!(lexer.next_token(), Some(Token::TextManipulationBasic(3))));
}

#[test]
fn test_z_camera_movement() {
    let mut lexer = Lexer::new("zzztzb");
    assert!(matches!(lexer.next_token(), Some(Token::CameraMovement)));
    assert!(matches!(lexer.next_token(), Some(Token::CameraMovement)));
    assert!(matches!(lexer.next_token(), Some(Token::CameraMovement)));
}

#[test]
fn test_unrecognized_g_prefix() {
    let mut lexer = Lexer::new("gx");
    assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "gx"));
}
```

### Acceptance Criteria
- All new tests pass
- No regressions
- `cargo test` succeeds

---

## Phase 4: Character-Consuming Commands

**Goal:** Implement commands that consume an arbitrary following character.

**File:** `vimscape_backend/src/lexer.rs`

### Prerequisites
- Phase 1 complete

### Tasks

- [x] 1. **Add `f`, `F`, `t`, `T` → `JumpToHorizontal`**
   - Each consumes the next character as the target
   - Example: `fa` → `JumpToHorizontal` (jump to 'a')
   - If no character follows, emit `Unhandled`

- [x] 2. **Add `r` → `TextManipulationBasic(n)`**
   - Consumes the next character as the replacement
   - Supports numeric prefix: `3ra` replaces 3 chars with 'a'
   - Example: `ra` → `TextManipulationBasic(1)`, `5rx` → `TextManipulationBasic(5)`

### Implementation Notes

These commands always consume exactly one following character. The character itself doesn't affect the token type, only whether the command is valid.

```rust
// When 'f', 'F', 't', or 'T' is encountered:
if let Some(_target_char) = self.input.next() {
    Some(Token::JumpToHorizontal)
} else {
    Some(Token::Unhandled("f".into())) // or the appropriate char
}
```

### Test Cases to Add

```rust
#[test]
fn test_jump_horizontal_f() {
    let mut lexer = Lexer::new("fa");
    assert!(matches!(lexer.next_token(), Some(Token::JumpToHorizontal)));
}

#[test]
fn test_jump_horizontal_all() {
    let mut lexer = Lexer::new("fxFytzT0");
    for _ in 0..4 {
        assert!(matches!(lexer.next_token(), Some(Token::JumpToHorizontal)));
    }
}

#[test]
fn test_replace_char() {
    let mut lexer = Lexer::new("ra5rx");
    assert!(matches!(lexer.next_token(), Some(Token::TextManipulationBasic(1))));
    assert!(matches!(lexer.next_token(), Some(Token::TextManipulationBasic(5))));
}

#[test]
fn test_jump_horizontal_incomplete() {
    let mut lexer = Lexer::new("f");
    assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "f"));
}
```

### Acceptance Criteria
- All new tests pass
- No regressions
- `cargo test` succeeds

---

## Phase 5: Operator-Pending Commands (d, y, c)

**Goal:** Implement delete, yank, and change operators that combine with motions.

**File:** `vimscape_backend/src/lexer.rs`

### Prerequisites
- Phases 1, 3, 4 complete (motion tokens must be parseable)

### Background

Operators (`d`, `y`, `c`) combine with motions:
- `dw` = delete word
- `dd` = delete line (doubled operator)
- `d3w` = delete 3 words
- `y$` = yank to end of line

### Tasks

1. **Add `d` operator handling → `DeleteText(n)`**
   - `dd` → `DeleteText(1)` (double-d means delete line)
   - `d` + motion → `DeleteText(n)` where n is motion count
   - `d$`, `d^`, `d0` → `DeleteText(1)`
   - `dw`, `dW`, `de`, `dE`, `db`, `dB` → `DeleteText(n)`
   - Numeric prefix applies: `3dd` → `DeleteText(3)`, `d3w` → `DeleteText(3)`

2. **Add `y` operator handling → `YankPaste`**
   - `yy` → `YankPaste` (yank line)
   - `y` + motion → `YankPaste`
   - `y$`, `yw`, `yiw`, `yaw` → `YankPaste`

3. **Add `c` operator handling → `TextManipulationAdvanced`**
   - `cc` → `TextManipulationAdvanced` (change line)
   - `c` + motion → `TextManipulationAdvanced`
   - `c$`, `cw`, `ciw`, `caw`, `ci)`, `ca)` → `TextManipulationAdvanced`
   - `s` → `TextManipulationAdvanced` (alias for `cl`)
   - `S` → `TextManipulationAdvanced` (alias for `cc`)
   - `C` → `TextManipulationAdvanced` (alias for `c$`)

### State Machine Update

Add new state:
```rust
enum State {
    None,
    AccumulatingCount(u32),
    OperatorPending { operator: char, count: u32 },
}
```

When in `OperatorPending`:
1. If next char is same as operator (e.g., `dd`), emit line operation
2. If next char is a motion, parse motion and emit combined token
3. If next char is digit, accumulate motion count
4. Handle text objects: `i`, `a` followed by `w`, `(`, `)`, `{`, `}`, `[`, `]`, `'`, `"`

### Test Cases to Add

```rust
#[test]
fn test_delete_line() {
    let mut lexer = Lexer::new("dd3dd");
    assert!(matches!(lexer.next_token(), Some(Token::DeleteText(1))));
    assert!(matches!(lexer.next_token(), Some(Token::DeleteText(3))));
}

#[test]
fn test_delete_motion() {
    let mut lexer = Lexer::new("dwdWd$d3w");
    assert!(matches!(lexer.next_token(), Some(Token::DeleteText(1))));
    assert!(matches!(lexer.next_token(), Some(Token::DeleteText(1))));
    assert!(matches!(lexer.next_token(), Some(Token::DeleteText(1))));
    assert!(matches!(lexer.next_token(), Some(Token::DeleteText(3))));
}

#[test]
fn test_yank() {
    let mut lexer = Lexer::new("yyywy$");
    assert!(matches!(lexer.next_token(), Some(Token::YankPaste)));
    assert!(matches!(lexer.next_token(), Some(Token::YankPaste)));
    assert!(matches!(lexer.next_token(), Some(Token::YankPaste)));
}

#[test]
fn test_change() {
    let mut lexer = Lexer::new("cccwc$");
    assert!(matches!(lexer.next_token(), Some(Token::TextManipulationAdvanced)));
    assert!(matches!(lexer.next_token(), Some(Token::TextManipulationAdvanced)));
    assert!(matches!(lexer.next_token(), Some(Token::TextManipulationAdvanced)));
}

#[test]
fn test_change_aliases() {
    let mut lexer = Lexer::new("sSC");
    assert!(matches!(lexer.next_token(), Some(Token::TextManipulationAdvanced)));
    assert!(matches!(lexer.next_token(), Some(Token::TextManipulationAdvanced)));
    assert!(matches!(lexer.next_token(), Some(Token::TextManipulationAdvanced)));
}

#[test]
fn test_text_objects() {
    let mut lexer = Lexer::new("ciwcawci)ca}");
    for _ in 0..4 {
        assert!(matches!(lexer.next_token(), Some(Token::TextManipulationAdvanced)));
    }
}
```

### Acceptance Criteria
- All new tests pass
- No regressions
- `cargo test` succeeds

---

## Phase 6: Command Mode Sequences

**Goal:** Implement command mode (`:`, `/`, `?`) with accumulation until terminator.

**File:** `vimscape_backend/src/lexer.rs`

### Prerequisites
- Phases 1, 2 complete (need `|enter|` and `<Esc>` handling)

### Background

Command mode sequences:
- Start with `:`, `/`, or `?`
- Accumulate characters until `|enter|` (completed) or `<Esc>` (cancelled)
- Special cases for `:h`, `:help`, `:w`

### Pipe-Delimited Special Keys

The Lua frontend sends:
- `|enter|` for Enter/Return
- `|tab|` for Tab
- `|backspace|` for Backspace
- `|space|` for Space

### Tasks

1. **Add pipe-delimited key parsing:**
   ```rust
   fn try_parse_pipe_delimited(&mut self) -> Option<String> {
       // If current char is '|', try to parse |enter|, |tab|, etc.
       // Return Some("enter"), Some("tab"), etc. if valid
       // Return None if not a valid pipe sequence (emit '|' as unhandled)
   }
   ```

2. **Add `/` and `?` → `CommandSearch(bool)`**
   - Accumulate until `|enter|` → `CommandSearch(true)`
   - Accumulate until `<Esc>` → `CommandSearch(false)`
   - Example: `/test|enter|` → `CommandSearch(true)`

3. **Add `:` → Various tokens based on content**
   - `:[digits]|enter|` → `JumpToLineNumber(digits)`
   - `:h` or `:help` → `HelpPage(bool)`
   - `:w` → `SaveFile(bool)`
   - Other commands → `Command(bool)`

### State Machine Update

Add new state:
```rust
enum State {
    None,
    AccumulatingCount(u32),
    OperatorPending { operator: char, count: u32 },
    CommandMode { start_char: char, content: String },
    SearchMode { start_char: char, content: String },
}
```

### Test Cases to Add

```rust
#[test]
fn test_search_completed() {
    let mut lexer = Lexer::new("/test|enter|");
    assert!(matches!(lexer.next_token(), Some(Token::CommandSearch(true))));
}

#[test]
fn test_search_cancelled() {
    let mut lexer = Lexer::new("/test<Esc>");
    assert!(matches!(lexer.next_token(), Some(Token::CommandSearch(false))));
}

#[test]
fn test_search_backward() {
    let mut lexer = Lexer::new("?pattern|enter|");
    assert!(matches!(lexer.next_token(), Some(Token::CommandSearch(true))));
}

#[test]
fn test_command_line_number() {
    let mut lexer = Lexer::new(":42|enter|");
    assert!(matches!(lexer.next_token(), Some(Token::JumpToLineNumber(ref s)) if s == "42"));
}

#[test]
fn test_help_page() {
    let mut lexer = Lexer::new(":h test|enter|:help topic<Esc>");
    assert!(matches!(lexer.next_token(), Some(Token::HelpPage(true))));
    assert!(matches!(lexer.next_token(), Some(Token::HelpPage(false))));
}

#[test]
fn test_save_file() {
    let mut lexer = Lexer::new(":w|enter|:w<Esc>");
    assert!(matches!(lexer.next_token(), Some(Token::SaveFile(true))));
    assert!(matches!(lexer.next_token(), Some(Token::SaveFile(false))));
}

#[test]
fn test_generic_command() {
    let mut lexer = Lexer::new(":Vimscape|enter|:q<Esc>");
    assert!(matches!(lexer.next_token(), Some(Token::Command(true))));
    assert!(matches!(lexer.next_token(), Some(Token::Command(false))));
}

#[test]
fn test_command_with_space() {
    let mut lexer = Lexer::new(":Vimscape|space|toggle|enter|");
    assert!(matches!(lexer.next_token(), Some(Token::Command(true))));
}
```

### Acceptance Criteria
- All new tests pass
- No regressions
- `cargo test` succeeds

---

## Phase 7: Replace Mode and Advanced Text Manipulation

**Goal:** Implement Replace mode (`R`) and `g~`, `gu`, `gU` operators.

**File:** `vimscape_backend/src/lexer.rs`

### Prerequisites
- Phases 1, 3, 5 complete

### Tasks

1. **Add `R` → `TextManipulationAdvanced`**
   - `R` enters replace mode
   - Accumulate characters until `<Esc>`
   - Example: `Rtext<Esc>` → `TextManipulationAdvanced`

2. **Add `g~`, `gu`, `gU` + motion → `TextManipulationAdvanced`**
   - These are operators like `d`, `y`, `c`
   - Take a motion argument
   - Example: `guw` → `TextManipulationAdvanced`, `gU3w` → `TextManipulationAdvanced`

### State Machine Update

Add new state:
```rust
ReplaceMode { content: String },
CaseOperatorPending { operator: String, count: u32 }, // "g~", "gu", "gU"
```

### Test Cases to Add

```rust
#[test]
fn test_replace_mode() {
    let mut lexer = Lexer::new("Rtest<Esc>");
    assert!(matches!(lexer.next_token(), Some(Token::TextManipulationAdvanced)));
}

#[test]
fn test_replace_mode_empty() {
    let mut lexer = Lexer::new("R<Esc>");
    assert!(matches!(lexer.next_token(), Some(Token::TextManipulationAdvanced)));
}

#[test]
fn test_case_toggle() {
    let mut lexer = Lexer::new("g~w");
    assert!(matches!(lexer.next_token(), Some(Token::TextManipulationAdvanced)));
}

#[test]
fn test_case_lower() {
    let mut lexer = Lexer::new("guw");
    assert!(matches!(lexer.next_token(), Some(Token::TextManipulationAdvanced)));
}

#[test]
fn test_case_upper() {
    let mut lexer = Lexer::new("gUw");
    assert!(matches!(lexer.next_token(), Some(Token::TextManipulationAdvanced)));
}

#[test]
fn test_case_with_count() {
    let mut lexer = Lexer::new("gu3w");
    assert!(matches!(lexer.next_token(), Some(Token::TextManipulationAdvanced)));
}
```

### Acceptance Criteria
- All new tests pass
- No regressions
- `cargo test` succeeds

---

## Phase 8: Special Sequences and Edge Cases

**Goal:** Handle remaining special cases and edge cases.

**File:** `vimscape_backend/src/lexer.rs`

### Prerequisites
- All previous phases complete

### Tasks

1. **Add `%` → `JumpFromContext`**
   - Single character, no lookahead needed
   - Note: In actual Vim, this expands to matchit call, but we just recognize `%`

2. **Handle `0` at line start**
   - `0` alone → `Unhandled("0")` (already implemented, verify behavior)
   - `0` in numeric prefix → not allowed (already implemented)

3. **Handle incomplete sequences gracefully:**
   - `g` at end of input → `Unhandled("g")`
   - `z` at end of input → `Unhandled("z")`
   - `d` at end of input → `Unhandled("d")`
   - `<C-` at end of input → `Unhandled("<C-")`

4. **Handle escape sequence `<Esc>`:**
   - Outside command mode → `Unhandled("<Esc>")` or ignore
   - Inside command/search mode → terminates with `false`

5. **Handle `Y` → `YankPaste`**
   - Alias for `y$` (yank to end of line)

### Test Cases to Add

```rust
#[test]
fn test_jump_from_context() {
    let mut lexer = Lexer::new("%");
    assert!(matches!(lexer.next_token(), Some(Token::JumpFromContext)));
}

#[test]
fn test_y_uppercase() {
    let mut lexer = Lexer::new("Y");
    assert!(matches!(lexer.next_token(), Some(Token::YankPaste)));
}

#[test]
fn test_incomplete_g() {
    let mut lexer = Lexer::new("g");
    assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "g"));
}

#[test]
fn test_incomplete_z() {
    let mut lexer = Lexer::new("z");
    assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "z"));
}

#[test]
fn test_incomplete_d() {
    let mut lexer = Lexer::new("d");
    assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "d"));
}

#[test]
fn test_escape_outside_command() {
    let mut lexer = Lexer::new("<Esc>");
    assert!(matches!(lexer.next_token(), Some(Token::Unhandled(_))));
}

#[test]
fn test_mixed_complex_sequence() {
    let mut lexer = Lexer::new("5j/test|enter|dd3gkzz:w|enter|");
    assert!(matches!(lexer.next_token(), Some(Token::MoveVerticalBasic(5))));
    assert!(matches!(lexer.next_token(), Some(Token::CommandSearch(true))));
    assert!(matches!(lexer.next_token(), Some(Token::DeleteText(1))));
    assert!(matches!(lexer.next_token(), Some(Token::MoveVerticalBasic(3))));
    assert!(matches!(lexer.next_token(), Some(Token::CameraMovement)));
    assert!(matches!(lexer.next_token(), Some(Token::SaveFile(true))));
}
```

### Acceptance Criteria
- All new tests pass
- No regressions
- `cargo test` succeeds
- All commented-out tests in lexer.rs can be uncommented and pass

---

## Summary

| Phase | Tokens Implemented | Complexity | Est. LOC |
|-------|-------------------|------------|----------|
| 1 | MoveHorizontalBasic, MoveHorizontalChunk (fix), UndoRedo, DotRepeat, JumpToVertical, YankPaste, TextManipulationBasic, JumpToLineNumber(G) | Low | ~50 |
| 2 | MoveVerticalChunk, JumpToVertical, CameraMovement, UndoRedo, WindowManagement (all `<C-X>`) | Medium | ~80 |
| 3 | MoveVerticalBasic(g), JumpToLineNumber(gg), TextManipulationBasic(gJ), CameraMovement(z) | Medium | ~60 |
| 4 | JumpToHorizontal, TextManipulationBasic(r) | Low | ~30 |
| 5 | DeleteText, YankPaste, TextManipulationAdvanced | High | ~150 |
| 6 | CommandSearch, Command, HelpPage, SaveFile, JumpToLineNumber(:n) | High | ~120 |
| 7 | TextManipulationAdvanced(R, g~/u/U) | Medium | ~80 |
| 8 | JumpFromContext, edge cases, Y alias | Low | ~40 |

**Total Estimated LOC:** ~610

---

## Execution Notes

1. Each phase should be executed in order
2. Run `cargo test` after each phase to verify no regressions
3. Phases can be parallelized only if explicitly noted as having no dependencies
4. Reference `SPEC.md` for token-to-skill mapping and XP values (implementation is in `parse_utils.rs`, outside scope of this plan)

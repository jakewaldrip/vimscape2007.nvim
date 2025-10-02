---
date: 2025-10-02T21:00:56+0000
git_commit: b294ec1aa609fb77e6ad21b2ae652db0d5975436
branch: main
repository: https://github.com/jakewaldrip/vimscape2007.nvim.git
topic: "Add numeric prefix handling for j/k movements in lexer"
tags: [research, codebase, lexer, parser, vim-motions, numeric-prefix, state-machine]
last_updated: 2025-10-02T21:00:56+0000
---

## Ticket Synopsis
FEATURE-001 requires enhancing the lexer in vimscape_backend to handle numeric prefixes for j and k vertical movement commands. When users input patterns like "16j", the lexer should parse this as a single token `Token::MoveVerticalBasic(16)` instead of treating the digits and command separately. The implementation should use `State::AccumulatingCount(u32)` for state tracking, cap values at 999, and extract digit accumulation logic into a helper method.

## Summary
The lexer has a solid foundation with `Peekable<Chars>` iterator and State enum infrastructure, but the state machine logic for numeric prefix handling is not yet implemented. The commented code at lines 26-33 shows the correct approach. The implementation requires:
1. Adding `State::AccumulatingCount(u32)` variant to the State enum
2. Implementing `accumulate_digits()` helper method
3. Using recursive calls for state transitions
4. Properly resetting state after consuming accumulated values
5. Following established Rust patterns from Skills and Token enums in the codebase

## Detailed Findings

### Lexer Architecture
- **Main implementation**: `vimscape_backend/src/lexer.rs` contains Lexer struct with Peekable<Chars<'a>> and State enum
- **Current state**: Only `State::None` variant exists (lines 5-7)
- **Token generation**: `next_token()` method at line 23 consumes characters and returns Option<Token>
- **Commented scaffolding**: Lines 26-33 show planned digit accumulation logic
- **Hardcoded counts**: Currently returns `Token::MoveVerticalBasic(1)` for all j/k movements (lines 36-37)

### Token System
- **Token definition**: `vimscape_backend/src/token.rs` defines `Token::MoveVerticalBasic(i32)` at line 5
- **Parameter meaning**: The i32 represents count/repetition, not direction
- **Token flow**: Tokens are consumed by `parse_utils::parse_action_into_skill()` which multiplies count by base experience
- **Unhandled tokens**: Non-j/k characters become `Token::Unhandled(String)` at line 38

### State Machine Patterns in Codebase
- **Enum with data**: Skills enum (`skills.rs:1-14`) shows pattern for enums containing values
- **Pattern matching**: `parse_utils.rs:5-9` demonstrates extracting values from enum variants
- **State transitions**: Need to check state BEFORE consuming characters, reset IMMEDIATELY after use
- **Iterator patterns**: `Peekable::peek()` for lookahead, `next()` for consumption

### Implementation Requirements
- **Digit accumulation**: Use `ch.is_ascii_digit()` and `ch.to_digit(10)` for parsing
- **Overflow handling**: Cap at 999, consume excess digits silently
- **Edge cases**: Handle standalone digits, leading zeros (treat as non-count), multiple commands
- **State flow**: None → AccumulatingCount(n) → None cycle per numeric prefix

### Test Infrastructure
- **Test module**: `lexer.rs:44-228` contains 1 active test and 31 commented test cases
- **Test patterns**: Uses standard Rust `#[cfg(test)]` and `#[test]` attributes
- **Planned tests**: Comprehensive test cases for various vim commands already outlined
- **Table-driven opportunity**: Could refactor to table-driven tests for better maintainability

## Code References
- `vimscape_backend/src/lexer.rs:5-7` - State enum definition
- `vimscape_backend/src/lexer.rs:10` - Peekable<Chars> field declaration
- `vimscape_backend/src/lexer.rs:23-41` - next_token() method implementation
- `vimscape_backend/src/lexer.rs:26-33` - Commented digit accumulation logic
- `vimscape_backend/src/lexer.rs:36-37` - Current hardcoded j/k token generation
- `vimscape_backend/src/token.rs:5` - MoveVerticalBasic token definition
- `vimscape_backend/src/parse_utils.rs:5-9` - Token to Skills conversion
- `vimscape_backend/src/skills.rs:1-14` - Skills enum pattern example
- `vimscape_backend/src/api.rs:20-29` - Token processing pipeline
- `vimscape_backend/src/lexer.rs:44-228` - Test structure and planned test cases

## Architecture Insights

### State Machine Design
The lexer should implement a two-state machine:
1. **State::None**: Default state, no digits accumulated
2. **State::AccumulatingCount(u32)**: Active accumulation with stored count

State transitions follow this pattern:
- None + digit → AccumulatingCount(n) + recurse
- AccumulatingCount(n) + command → None + return Token with count n
- AccumulatingCount(n) + other → None + handle appropriately

### Recommended Implementation Pattern
```rust
pub fn next_token(&mut self) -> Option<Token> {
    // Check state FIRST
    match self.state {
        State::AccumulatingCount(count) => {
            let ch = self.input.next()?;
            self.state = State::None; // Reset immediately
            return match ch {
                'j' | 'k' => Some(Token::MoveVerticalBasic(count as i32)),
                _ => self.next_token(), // Recurse for standalone digits
            };
        }
        State::None => { /* continue normal processing */ }
    }
    
    let ch = self.input.next()?;
    if ch.is_ascii_digit() {
        let count = self.accumulate_digits(ch);
        self.state = State::AccumulatingCount(count);
        return self.next_token(); // Recursive call
    }
    
    // Normal token matching
    match ch {
        'j' | 'k' => Some(Token::MoveVerticalBasic(1)),
        _ => Some(Token::Unhandled(ch.into())),
    }
}
```

### Helper Method Design
```rust
fn accumulate_digits(&mut self, first_digit: char) -> u32 {
    let mut count = first_digit.to_digit(10).unwrap();
    
    while let Some(&ch) = self.input.peek() {
        if !ch.is_ascii_digit() { break; }
        self.input.next(); // Consume
        
        let digit = ch.to_digit(10).unwrap();
        count = count.saturating_mul(10).saturating_add(digit);
        
        if count > 999 {
            // Consume excess digits
            while let Some(&ch) = self.input.peek() {
                if !ch.is_ascii_digit() { break; }
                self.input.next();
            }
            return 999;
        }
    }
    count
}
```

### Key Design Decisions
1. **Use recursive calls** for state transitions (max depth: 1, clean code)
2. **Reset state immediately** after consuming to prevent state leakage
3. **Check state before consuming** to handle accumulated counts properly
4. **Use saturating arithmetic** to prevent panics on overflow
5. **Peek for lookahead** without consuming, essential for digit accumulation

## Historical Context (from notes/)
- `notes/tickets/feature_lexer_numeric_prefix.md` - Comprehensive feature specification created 2025-01-02
  - Detailed requirements including 999 cap and edge case handling
  - Keywords and patterns identified for implementation
  - Success criteria with specific test cases
  - Design decisions already made (State::AccumulatingCount variant, helper method extraction)

## Related Research
None yet - this is the first research document for the lexer numeric prefix feature.

## Open Questions
1. **Leading zero handling**: Should "0j" be treated as Token::Unhandled or as movement with count 0?
2. **Standalone digits**: Should "123" without command return Token::Unhandled or be silently consumed?
3. **Error reporting**: Should invalid sequences like "999x" generate error tokens or just Unhandled?
4. **State enum visibility**: Should State enum derive Debug for testing as recommended in ticket?
5. **Test migration**: Should the 31 commented tests be converted to table-driven format?
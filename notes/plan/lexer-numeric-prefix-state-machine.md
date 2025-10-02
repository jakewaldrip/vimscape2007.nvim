# Lexer Numeric Prefix State Machine Implementation Plan

## Overview

Enhance the lexer in vimscape_backend to handle numeric prefixes for j, k, and future vertical movement commands using a proper state machine approach. The implementation will use `State::AccumulatingCount(u32)` to track accumulated digits and generate appropriate tokens based on the following character.

## Current State Analysis

The lexer currently uses a lookahead approach with `peek_for_numeric_command` method (lines 25-55) which works but doesn't utilize the state machine infrastructure. The State enum has `AccumulatingCount(u32)` variant but it's unused. This approach makes it difficult to extend for future commands and doesn't follow the intended state machine pattern.

## Desired End State

After implementation, the lexer will:
- Use `State::AccumulatingCount(u32)` to maintain accumulated digit state
- Parse "16j" as `Token::MoveVerticalBasic(16)`
- Parse standalone "123" as a single `Token::Unhandled("123")`
- Be easily extendable for future commands that use numeric prefixes
- Handle edge cases like 999+ digit sequences with proper capping
- Use a clean state machine pattern without lookahead

### Key Discoveries:
- Current lookahead implementation at `vimscape_backend/src/lexer.rs:25-55`
- State enum already has AccumulatingCount variant at `vimscape_backend/src/lexer.rs:8`
- Token enum supports count parameter at `vimscape_backend/src/token.rs:5`
- Tests already exist and pass with current implementation (lines 101-167)
- Parse utils multiplies count by base experience at `vimscape_backend/src/parse_utils.rs:5-8`

## What We're NOT Doing

- Using lookahead or peek-based approaches
- Handling commands beyond j and k in this implementation
- Processing leading zeros as part of count (0j = Unhandled("0") + MoveVerticalBasic(1))
- Creating multiple unhandled tokens for standalone digits (should be single token)
- Modifying Token enum structure or parse_utils logic

## Implementation Approach

Use a true state machine where:
1. Digits transition to `AccumulatingCount` state and continue accumulating
2. When a non-digit is encountered while in `AccumulatingCount`:
   - If it's j/k: generate `MoveVerticalBasic` with the count
   - Otherwise: generate `Unhandled` with the accumulated digit string
3. State is maintained across `next_token()` calls, not reset immediately
4. Helper method handles digit accumulation logic

## Phase 1: Remove Lookahead Infrastructure

### Overview
Remove the existing lookahead implementation to prepare for state machine approach.

### Changes Required:

#### 1. Remove peek_for_numeric_command Method
**File**: `vimscape_backend/src/lexer.rs`
**Changes**: Delete lines 25-55 (the entire peek_for_numeric_command method)

### Success Criteria:

#### Manual Verification:
- [x] Method removed completely
- [x] No references to peek_for_numeric_command remain
- [x] Code still compiles (tests will fail temporarily)

---

## Phase 2: Add State Machine Helper Methods

### Overview
Create helper methods for digit accumulation and state management.

### Changes Required:

#### 1. Add accumulated_string Field to Lexer
**File**: `vimscape_backend/src/lexer.rs`
**Changes**: Modify Lexer struct at lines 11-14

```rust
pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    state: State,
    accumulated_string: String,
}
```

#### 2. Update Lexer::new Constructor
**File**: `vimscape_backend/src/lexer.rs`
**Changes**: Update constructor at lines 17-23

```rust
pub fn new(input: &'a str) -> Self {
    let chars = input.chars().peekable();
    Self {
        input: chars,
        state: State::None,
        accumulated_string: String::new(),
    }
}
```

#### 3. Add accumulate_digit Method
**File**: `vimscape_backend/src/lexer.rs`
**Changes**: Add new method after the constructor

```rust
fn accumulate_digit(&mut self, digit: char) -> u32 {
    self.accumulated_string.push(digit);
    
    // Parse the accumulated string to get current count
    let mut count: u32 = 0;
    for ch in self.accumulated_string.chars() {
        let digit_value = ch.to_digit(10).unwrap_or(0);
        count = count.saturating_mul(10).saturating_add(digit_value);
        if count > 999 {
            count = 999;
            break;
        }
    }
    
    count
}
```

### Success Criteria:

#### Manual Verification:
- [x] Helper method correctly accumulates digits
- [x] String field tracks accumulated digits
- [x] Count calculation respects 999 cap
- [x] No overflow panics

---

## Phase 3: Implement State Machine Logic

### Overview
Replace the current next_token implementation with proper state machine logic.

### Changes Required:

#### 1. Rewrite next_token Method
**File**: `vimscape_backend/src/lexer.rs`
**Changes**: Replace the entire next_token method at lines 57-78

```rust
pub fn next_token(&mut self) -> Option<Token> {
    // Handle accumulated state from previous calls
    match self.state {
        State::AccumulatingCount(count) => {
            // We're in accumulating state, check next character
            if let Some(&ch) = self.input.peek() {
                if ch.is_ascii_digit() {
                    // Continue accumulating
                    self.input.next();
                    let new_count = self.accumulate_digit(ch);
                    self.state = State::AccumulatingCount(new_count);
                    return self.next_token(); // Recurse to continue processing
                } else {
                    // Non-digit encountered, process based on what it is
                    self.state = State::None;
                    let accumulated = self.accumulated_string.clone();
                    self.accumulated_string.clear();
                    
                    match ch {
                        'j' | 'k' => {
                            self.input.next(); // Consume the command
                            return Some(Token::MoveVerticalBasic(count as i32));
                        }
                        _ => {
                            // Not a command we handle with counts
                            return Some(Token::Unhandled(accumulated));
                        }
                    }
                }
            } else {
                // End of input while accumulating
                self.state = State::None;
                let accumulated = self.accumulated_string.clone();
                self.accumulated_string.clear();
                return Some(Token::Unhandled(accumulated));
            }
        }
        State::None => {
            // Normal processing
            let ch = self.input.next()?;
            
            // Check for digits (excluding leading zero)
            if ch.is_ascii_digit() && ch != '0' {
                let count = self.accumulate_digit(ch);
                self.state = State::AccumulatingCount(count);
                return self.next_token(); // Recurse to process next
            }
            
            // Regular token processing
            match ch {
                'j' | 'k' => Some(Token::MoveVerticalBasic(1)),
                _ => Some(Token::Unhandled(ch.into())),
            }
        }
    }
}
```

### Success Criteria:

#### Manual Verification:
- [x] "16j" produces `Token::MoveVerticalBasic(16)`
- [x] "j" produces `Token::MoveVerticalBasic(1)`
- [x] "123" produces single `Token::Unhandled("123")`
- [x] "0j" produces `Token::Unhandled("0")` then `Token::MoveVerticalBasic(1)`
- [x] "1000j" produces `Token::MoveVerticalBasic(999)`
- [x] State properly maintained across calls
- [x] Accumulated string properly cleared after use

---

## Phase 4: Handle Overflow Edge Cases

### Overview
Ensure proper handling of numbers exceeding 999 by consuming excess digits.

### Changes Required:

#### 1. Update accumulate_digit Method
**File**: `vimscape_backend/src/lexer.rs`
**Changes**: Enhance the accumulate_digit method to consume excess digits

```rust
fn accumulate_digit(&mut self, digit: char) -> u32 {
    self.accumulated_string.push(digit);
    
    // Parse the accumulated string to get current count
    let mut count: u32 = 0;
    for ch in self.accumulated_string.chars() {
        let digit_value = ch.to_digit(10).unwrap_or(0);
        let new_count = count.saturating_mul(10).saturating_add(digit_value);
        if new_count > 999 {
            // Cap at 999 but continue accumulating the string
            count = 999;
            
            // Consume any remaining digits
            while let Some(&ch) = self.input.peek() {
                if ch.is_ascii_digit() {
                    self.input.next();
                    self.accumulated_string.push(ch);
                } else {
                    break;
                }
            }
            return count;
        }
        count = new_count;
    }
    
    count
}
```

### Success Criteria:

#### Manual Verification:
- [x] "1234567j" produces `Token::MoveVerticalBasic(999)`
- [x] "9999k" produces `Token::MoveVerticalBasic(999)`
- [x] Excess digits consumed and not left for next token
- [x] Accumulated string contains all digits for unhandled case

---

## Phase 5: Test Validation

### Overview
Ensure all existing tests pass with the new state machine implementation.

### Changes Required:

#### 1. Fix Expected Behavior for Standalone Digits Test
**File**: `vimscape_backend/src/lexer.rs`
**Changes**: Update test at lines 136-141 to expect single unhandled token

```rust
#[test]
fn test_standalone_digits() {
    let mut lexer = Lexer::new("123");
    assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "123"));
    assert!(matches!(lexer.next_token(), None));
}
```

#### 2. Add Test for Large Standalone Digits
**File**: `vimscape_backend/src/lexer.rs`
**Changes**: Add new test after test_standalone_digits

```rust
#[test]
fn test_standalone_digits_overflow() {
    let mut lexer = Lexer::new("999999");
    assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "999999"));
    assert!(matches!(lexer.next_token(), None));
}

#[test]
fn test_mixed_digits_and_commands() {
    let mut lexer = Lexer::new("12x34j");
    assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "12"));
    assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "x"));
    assert!(matches!(lexer.next_token(), Some(Token::MoveVerticalBasic(34))));
    assert!(matches!(lexer.next_token(), None));
}
```

### Success Criteria:

#### Manual Verification:
- [x] All existing tests pass (except standalone_digits which needs update)
- [x] New tests validate edge cases
- [x] No regressions in functionality
- [x] State machine properly resets between test cases

---

## Testing Strategy

### Unit Tests:
- Basic numeric prefix parsing (5j, 123k)
- Boundary case at 999
- Overflow handling (1000+ produces 999)
- No prefix cases (plain j/k)
- Standalone digits as single token
- Leading zero handling
- Multiple commands in sequence
- Mixed input patterns with non-command characters

### Integration Tests:
- Full input strings with multiple command types
- Verify tokens are consumed correctly by parse_utils
- Ensure experience calculation works with numeric prefixes

### Manual Testing Steps:
1. Run `cargo test` in vimscape_backend directory
2. Test with various input patterns via the API
3. Verify state machine transitions correctly
4. Check that standalone digits produce single token
5. Validate extensibility by adding new command support

## Performance Considerations

- Using `saturating_mul` and `saturating_add` prevents overflow panics
- Single recursion per digit group keeps stack usage minimal
- String accumulation allows proper handling of standalone digits
- State maintained across calls reduces redundant processing

## Migration Notes

This replaces the lookahead implementation with a proper state machine. The external behavior remains the same except for standalone digits which now produce a single `Token::Unhandled` containing all digits instead of multiple tokens. This is more efficient and aligns with the extensibility requirement.

## Future Extensions

The state machine approach makes it trivial to add support for new commands:
1. Add new command characters to the match statement in the AccumulatingCount branch
2. Generate appropriate token type with the accumulated count
3. No changes needed to digit accumulation logic

Example for adding 'h' and 'l' horizontal movement:
```rust
match ch {
    'j' | 'k' => Some(Token::MoveVerticalBasic(count as i32)),
    'h' | 'l' => Some(Token::MoveHorizontalBasic(count as i32)),
    _ => Some(Token::Unhandled(accumulated))
}
```

## References

- Original ticket: `notes/tickets/feature_lexer_numeric_prefix.md`
- Related research: `notes/research/2025-01-02_lexer_numeric_prefix.md`
- Previous implementation: `notes/plan/lexer-numeric-prefix.md` (used lookahead approach)
- Current code: `vimscape_backend/src/lexer.rs:25-78` (peek_for_numeric_command)
- Token processing: `vimscape_backend/src/parse_utils.rs:5-8`
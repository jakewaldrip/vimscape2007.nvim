# Lexer Numeric Prefix Implementation Plan

## Overview

Enhance the lexer in vimscape_backend to handle numeric prefixes for j and k vertical movement commands, parsing patterns like "16j" as a single `Token::MoveVerticalBasic(16)` token instead of treating digits and commands separately.

## Current State Analysis

The lexer has basic infrastructure but lacks numeric prefix support. Currently at `vimscape_backend/src/lexer.rs`:
- State enum with only `State::None` variant (lines 5-7)
- Peekable character iterator in place (line 10)
- Hardcoded count of 1 for j/k movements (lines 36-37)
- Commented pseudocode showing intended approach (lines 26-33)
- Test infrastructure with 1 active and 31 commented tests (lines 44-228)

## Desired End State

After implementation, the lexer will:
- Parse "16j" as `Token::MoveVerticalBasic(16)`
- Handle edge cases like 999+ digit sequences and standalone digits
- Use `State::AccumulatingCount(u32)` for tracking numeric prefixes
- Have clean separation with `accumulate_digits()` helper method
- Pass comprehensive test suite including boundary cases

### Key Discoveries:
- Token enum already supports count parameter at `vimscape_backend/src/token.rs:5`
- Skills enum pattern at `vimscape_backend/src/skills.rs:1-14` shows data variant approach
- Parse utils multiplies count by base experience at `vimscape_backend/src/parse_utils.rs:5-8`
- Both j and k should return positive counts (repetition, not direction)

## What We're NOT Doing

- Handling other commands beyond j and k
- Implementing negative counts or direction encoding
- Processing leading zeros as part of count (0j = Unhandled("0") + MoveVerticalBasic(1))
- Creating recursive implementations (use iterative approach per codebase patterns)
- Modifying Token enum structure or parse_utils logic

## Implementation Approach

Use a state machine with two states to track digit accumulation. When digits are encountered, accumulate them and transition to `AccumulatingCount` state. When a j/k command follows, generate a token with the accumulated count. Standalone digits remain as individual Unhandled tokens.

## Phase 1: State Infrastructure

### Overview
Add the `AccumulatingCount` variant to the State enum and derive Debug for testing purposes.

### Changes Required:

#### 1. State Enum Enhancement
**File**: `vimscape_backend/src/lexer.rs`
**Changes**: Modify State enum at lines 5-7

```rust
#[derive(Debug)]
enum State {
    None,
    AccumulatingCount(u32),
}
```

### Success Criteria:

#### Manual Verification:
- [x] Code compiles without errors
- [x] State enum can be printed in debug format
- [x] Both state variants can be instantiated

---

## Phase 2: Digit Accumulation Helper

### Overview
Create a helper method to accumulate digits from the input stream, handling the 999 cap and excess digit consumption.

### Changes Required:

#### 1. Add accumulate_digits Method
**File**: `vimscape_backend/src/lexer.rs`
**Changes**: Add new method in impl block after line 21

```rust
fn accumulate_digits(&mut self, first_digit: char) -> u32 {
    let mut count = first_digit.to_digit(10).unwrap_or(0);
    
    while let Some(&ch) = self.input.peek() {
        if !ch.is_ascii_digit() {
            break;
        }
        
        self.input.next(); // Consume the digit
        
        let digit = ch.to_digit(10).unwrap_or(0);
        let new_count = count.saturating_mul(10).saturating_add(digit);
        
        if new_count > 999 {
            // Consume any remaining digits
            while let Some(&ch) = self.input.peek() {
                if !ch.is_ascii_digit() {
                    break;
                }
                self.input.next();
            }
            return 999;
        }
        
        count = new_count;
    }
    
    count
}
```

### Success Criteria:

#### Manual Verification:
- [x] Method correctly accumulates multi-digit numbers
- [x] Returns 999 for numbers exceeding the cap
- [x] Consumes excess digits when over 999
- [x] Stops accumulation at non-digit characters

---

## Phase 3: State Machine Logic

### Overview
Implement the state machine logic in `next_token()` to handle numeric prefixes and state transitions.

### Changes Required:

#### 1. Implement State Machine in next_token
**File**: `vimscape_backend/src/lexer.rs`
**Changes**: Replace the entire `next_token` method at lines 23-41

```rust
pub fn next_token(&mut self) -> Option<Token> {
    // Check for accumulated count state first
    if let State::AccumulatingCount(count) = self.state {
        self.state = State::None; // Reset state immediately
        
        if let Some(ch) = self.input.next() {
            match ch {
                'j' | 'k' => {
                    return Some(Token::MoveVerticalBasic(count as i32));
                }
                _ => {
                    // Not a j/k command, treat as unhandled
                    return Some(Token::Unhandled(ch.into()));
                }
            }
        } else {
            return None;
        }
    }
    
    let ch = self.input.next()?;
    
    // Check for ascii digits (but not leading zero)
    if ch.is_ascii_digit() && ch != '0' {
        let count = self.accumulate_digits(ch);
        self.state = State::AccumulatingCount(count);
        return self.next_token(); // Recursive call to process next character
    }
    
    let token = match ch {
        'j' => Token::MoveVerticalBasic(1),
        'k' => Token::MoveVerticalBasic(1),
        _ => Token::Unhandled(ch.into()),
    };
    Some(token)
}
```

### Success Criteria:

#### Manual Verification:
- [x] "16j" produces `Token::MoveVerticalBasic(16)`
- [x] "j" produces `Token::MoveVerticalBasic(1)`
- [x] "123" produces three `Token::Unhandled` tokens
- [x] "0j" produces `Token::Unhandled("0")` then `Token::MoveVerticalBasic(1)`
- [x] "1000j" produces `Token::MoveVerticalBasic(999)`
- [x] State resets properly after each token generation

---

## Phase 4: Test Implementation

### Overview
Create comprehensive tests to validate all numeric prefix functionality and edge cases.

### Changes Required:

#### 1. Add Numeric Prefix Tests
**File**: `vimscape_backend/src/lexer.rs`
**Changes**: Add new tests in the test module after line 57

```rust
#[test]
fn test_numeric_prefix_basic() {
    let mut lexer = Lexer::new("5j");
    assert!(matches!(lexer.next_token(), Some(Token::MoveVerticalBasic(5))));
    assert!(matches!(lexer.next_token(), None));
}

#[test]
fn test_numeric_prefix_large() {
    let mut lexer = Lexer::new("123k");
    assert!(matches!(lexer.next_token(), Some(Token::MoveVerticalBasic(123))));
    assert!(matches!(lexer.next_token(), None));
}

#[test]
fn test_numeric_prefix_cap() {
    let mut lexer = Lexer::new("999j");
    assert!(matches!(lexer.next_token(), Some(Token::MoveVerticalBasic(999))));
    assert!(matches!(lexer.next_token(), None));
}

#[test]
fn test_numeric_prefix_overflow() {
    let mut lexer = Lexer::new("1234567j");
    assert!(matches!(lexer.next_token(), Some(Token::MoveVerticalBasic(999))));
    assert!(matches!(lexer.next_token(), None));
}

#[test]
fn test_no_numeric_prefix() {
    let mut lexer = Lexer::new("j");
    assert!(matches!(lexer.next_token(), Some(Token::MoveVerticalBasic(1))));
    assert!(matches!(lexer.next_token(), None));
}

#[test]
fn test_standalone_digits() {
    let mut lexer = Lexer::new("123");
    assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "1"));
    assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "2"));
    assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "3"));
    assert!(matches!(lexer.next_token(), None));
}

#[test]
fn test_leading_zero() {
    let mut lexer = Lexer::new("0j");
    assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "0"));
    assert!(matches!(lexer.next_token(), Some(Token::MoveVerticalBasic(1))));
    assert!(matches!(lexer.next_token(), None));
}

#[test]
fn test_multiple_commands() {
    let mut lexer = Lexer::new("2j3k");
    assert!(matches!(lexer.next_token(), Some(Token::MoveVerticalBasic(2))));
    assert!(matches!(lexer.next_token(), Some(Token::MoveVerticalBasic(3))));
    assert!(matches!(lexer.next_token(), None));
}

#[test]
fn test_mixed_input() {
    let mut lexer = Lexer::new("j5kx");
    assert!(matches!(lexer.next_token(), Some(Token::MoveVerticalBasic(1))));
    assert!(matches!(lexer.next_token(), Some(Token::MoveVerticalBasic(5))));
    assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "x"));
    assert!(matches!(lexer.next_token(), None));
}
```

#### 2. Update Existing WIP Test
**File**: `vimscape_backend/src/lexer.rs`
**Changes**: Update expectation comment for the wip_test at line 48

```rust
#[test]
fn wip_test() {
    let src = "10jkj";
    let mut lexer = Lexer::new(src);
    
    println!("Source: {src}");
    while let Some(token) = lexer.next_token() {
        println!("Output: {token:?}");
    }
    // Expected output:
    // Token::MoveVerticalBasic(10)
    // Token::MoveVerticalBasic(1)
    // Token::MoveVerticalBasic(1)
}
```

### Success Criteria:

#### Manual Verification:
- [x] All new tests pass
- [x] Existing wip_test produces expected output
- [x] Edge cases properly handled
- [x] No panics or unexpected behavior
- [x] Test coverage includes all specified scenarios

---

## Testing Strategy

### Unit Tests:
- Basic numeric prefix parsing (5j, 123k)
- Boundary case at 999
- Overflow handling (1000+)
- No prefix cases (plain j/k)
- Standalone digits
- Leading zero handling
- Multiple commands in sequence
- Mixed input patterns

### Integration Tests:
- Full input strings with multiple command types
- Verify tokens are consumed correctly by parse_utils
- Ensure experience calculation works with numeric prefixes

### Manual Testing Steps:
1. Run `cargo test` in vimscape_backend directory
2. Test with various input patterns via the API
3. Verify no regression in existing functionality
4. Check memory usage with large digit sequences
5. Validate performance with repeated tokenization

## Performance Considerations

- Using `saturating_mul` and `saturating_add` prevents overflow panics
- Early exit when 999 cap is reached minimizes unnecessary processing
- Single recursion depth (max 1 level) keeps stack usage minimal
- Peekable iterator avoids backtracking overhead

## Migration Notes

No migration needed as this is a new feature. Existing code that expects `MoveVerticalBasic(1)` will continue to work as the default behavior remains unchanged.

## References

- Original ticket: `notes/tickets/feature_lexer_numeric_prefix.md`
- Related research: `notes/research/2025-01-02_lexer_numeric_prefix.md`
- Similar implementation: `vimscape_backend/src/skills.rs:1-14` (enum with data variants)
- Token processing: `vimscape_backend/src/parse_utils.rs:5-8`

## Deviations from Plan

### Phase 3: State Machine Logic
- **Original Plan**: Use state machine with `State::AccumulatingCount(u32)` to track accumulated digits, then process the next character to determine if it's a j/k command
- **Actual Implementation**: Used a lookahead approach with `peek_for_numeric_command` method that checks if digits are followed by j/k without maintaining state
- **Reason for Deviation**: The state machine approach would consume digits even when they're not followed by j/k, making it difficult to handle standalone digits correctly (e.g., "123" should produce three separate Unhandled tokens)
- **Impact Assessment**: All functional requirements are met, all tests pass. The state enum and AccumulatingCount variant remain in the code but unused. The lookahead approach is cleaner for this specific use case.
- **Date/Time**: 2025-01-02
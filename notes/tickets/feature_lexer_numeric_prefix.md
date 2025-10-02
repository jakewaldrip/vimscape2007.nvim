---
type: feature
priority: medium
created: 2025-01-02T10:00:00Z
status: implemented
tags: [lexer, parser, vim-motions, numeric-prefix]
keywords: [Lexer, next_token, Token, MoveVerticalBasic, State, AccumulatingCount, peek, peekable, is_ascii_digit]
patterns: [state-machine, digit-accumulation, token-generation, peekable-iterator]
research_document: notes/research/2025-01-02_lexer_numeric_prefix.md
---

# FEATURE-001: Add numeric prefix handling for j/k movements in lexer

## Description
Enhance the lexer in vimscape_backend to handle numeric prefixes for j and k vertical movement commands. When users input patterns like "16j", the lexer should parse this as a single token `Token::MoveVerticalBasic(16)` instead of treating the digits and command separately.

## Context
The lexer currently handles basic j and k commands but doesn't support the vim-standard numeric prefix pattern for repeating movements. This is a fundamental vim feature where users can prefix movement commands with numbers to repeat them multiple times. The implementation should leverage the existing State enum for tracking the accumulation of digits.

## Requirements

### Functional Requirements
- Parse numeric prefixes (1-999) preceding j or k commands
- Generate `Token::MoveVerticalBasic(count)` with the appropriate count value
- Handle edge cases like standalone digits without terminating commands
- Support multiple numeric+command sequences in a single input string (e.g., "1j2k" → two separate tokens)
- Preserve default behavior where plain "j" or "k" generates count of 1
- Cap numeric values at 999 to prevent overflow
- Consume and silently discard any digits beyond the 999 limit

### Non-Functional Requirements
- Implement using `State::AccumulatingCount(u32)` variant in the State enum
- Extract digit accumulation logic into a helper method for code organization
- State enum should derive Debug for testing purposes
- Create table-driven tests for comprehensive validation
- Do not handle leading zeros as part of the count (e.g., "0" is not a digit in this context)
- Maintain backward compatibility with existing token generation

## Current State
- Lexer exists with basic j/k handling that always returns `Token::MoveVerticalBasic(1)`
- State enum exists but only has `State::None` variant
- Framework for digit detection is partially implemented (lines 26-33 in lexer.rs)
- Token enum already supports count parameter in `MoveVerticalBasic(i32)` variant

## Desired State
- Lexer successfully parses patterns like "16j" into `Token::MoveVerticalBasic(16)`
- State machine properly tracks digit accumulation via `State::AccumulatingCount(u32)`
- Clean separation of concerns with `accumulate_digits()` helper method
- Comprehensive test coverage including edge cases

## Research Context

### Keywords to Search
- `Lexer` - Main lexer implementation
- `next_token` - Token generation method to modify
- `Token` - Token enum definition
- `MoveVerticalBasic` - Specific token variant being enhanced
- `State` - State enum needing new variant
- `AccumulatingCount` - New state variant to add
- `peek`/`peekable` - Iterator methods for lookahead
- `is_ascii_digit` - Character checking method

### Patterns to Investigate
- state-machine - How to implement state transitions
- digit-accumulation - Pattern for building up numeric values
- token-generation - Current token creation patterns
- peekable-iterator - Usage of Peekable<Chars> for lookahead

### Key Decisions Made
- Use `State::AccumulatingCount(u32)` for tracking numeric prefix
- Cap at 999 to prevent overflow issues
- Consume excess digits silently
- Extract logic to `accumulate_digits()` helper
- Use table-driven tests
- Default count remains 1 when no prefix
- "0" handled separately, not as part of count
- Maintain `Token::MoveVerticalBasic` naming

## Success Criteria

### Automated Verification
- [ ] Unit tests pass for basic cases ("j" → 1, "5j" → 5, "16j" → 16)
- [ ] Edge case tests pass (999j, 1000j caps at 999, standalone digits)
- [ ] Table-driven tests validate all input/output combinations
- [ ] Multiple commands parsed correctly ("1j2k3j")
- [ ] Test for exact boundary at 999

### Manual Verification
- [ ] Lexer correctly parses numeric prefixes in interactive testing
- [ ] State transitions happen immediately after consuming j/k
- [ ] No regression in existing j/k handling
- [ ] Helper method `accumulate_digits()` is clean and maintainable

## Related Information
- Token enum definition in `vimscape_backend/src/token.rs`
- Existing test structure in lexer.rs (lines 44-228)
- Comment at line 26-33 shows initial attempt at digit handling

## Notes
- Both j and k return positive counts (the value represents repetition count, not direction)
- Space or any non-digit character breaks accumulation
- This implementation focuses only on j/k; other commands are out of scope
- Leading zeros should be handled separately in future iterations
- Pattern "1j2k" results in two separate tokens, not an error

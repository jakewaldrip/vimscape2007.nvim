//! Vim Command Lexer
//!
//! This module implements a state machine lexer that tokenizes Vim-style command sequences
//! into discrete tokens for processing by the game engine.
//!
//! # State Machine Overview
//!
//! The lexer operates using the following states:
//! - `State::None` - Initial state, awaiting first character of a command
//! - `State::AccumulatingCount` - Parsing numeric prefix (e.g., "123" in "123j")
//! - `State::OperatorPending` - Awaiting motion after operator (d, y, c)
//! - `State::CaseOperatorPending` - Awaiting motion after case operator (g~, gu, gU)
//! - `State::CommandMode` - Parsing Ex command after ':'
//! - `State::SearchMode` - Parsing search pattern after '/' or '?'
//! - `State::ReplaceMode` - Parsing replacement character after 'r'
//!
//! # Intentionally Unhandled Commands
//!
//! The following command categories are intentionally returned as `Token::Unhandled`:
//!
//! - **Insert mode commands** (`o`, `O`, `i`, `I`, `a`, `A`): These enter insert mode,
//!   which is not tracked by this lexer since the game handles text input separately.
//!
//! - **Visual mode commands** (`v`, `V`, `<C-V>`): Visual mode selection is a future
//!   consideration per the project specification.
//!
//! - **Macro commands** (`q`, `@`): Macro recording and playback is a future
//!   consideration per the project specification.
//!
//! - **Window split commands** (`:sp`, `:vs`): Window management beyond basic
//!   navigation is handled elsewhere.

use std::{iter::Peekable, str::Chars};

use crate::token::Token;

#[derive(Debug, Clone, Copy)]
enum Operator {
    Delete, // d
    Yank,   // y
    Change, // c
}

#[derive(Debug)]
enum State {
    None,
    AccumulatingCount(u32),
    OperatorPending { operator: Operator, count: u32 },
    CommandMode { content: String },
    SearchMode { content: String },
    ReplaceMode { content: String },
    CaseOperatorPending { operator: String, count: u32 }, // "g~", "gu", "gU"
}

/// Result of handling a simple command character
enum CommandResult {
    /// Return this token immediately
    Token(Token),
    /// Need to consume next char, if present return first token, else second
    ConsumeNextOptional(Token, Token),
    /// Not a simple command - needs special handling
    NotSimple,
}

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    state: State,
    accumulated_string: String,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.chars().peekable(),
            state: State::None,
            accumulated_string: String::new(),
        }
    }

    /// Try to parse a control sequence like `<C-X>`.
    /// Returns Some(char) with the control character (e.g., 'U', 'D', 'F', etc.) if valid.
    /// Returns None if not a valid control sequence.
    /// The caller has already consumed the initial '<'.
    fn try_parse_control_sequence(&mut self) -> Option<char> {
        if self.input.peek() != Some(&'C') {
            return None;
        }
        self.input.next();

        if self.input.peek() != Some(&'-') {
            return None;
        }
        self.input.next();

        let ctrl_char = self.input.next()?;

        if self.input.peek() != Some(&'>') {
            return None;
        }
        self.input.next();

        Some(ctrl_char)
    }

    /// Handle a control sequence and return the appropriate token.
    /// `count` is the numeric prefix (default 1).
    fn handle_control_sequence(&mut self, ctrl_char: char, count: u32) -> Token {
        match ctrl_char {
            'U' | 'D' => Token::MoveVerticalChunk(i32::try_from(count).unwrap()),
            'F' | 'B' => Token::JumpToVertical,
            'E' | 'Y' => Token::CameraMovement,
            'R' => Token::UndoRedo,
            'H' | 'J' | 'K' | 'L' => Token::WindowManagement,
            'W' => {
                let _ = self.input.next();
                Token::WindowManagement
            }
            _ => Token::Unhandled(format!("<C-{ctrl_char}>")),
        }
    }

    /// Try to parse a pipe-delimited special key like `|enter|`, `|tab|`, etc.
    /// Returns `Some(key_name)` if valid (e.g., "enter", "tab", "`backspace`", "space").
    /// Returns None if not a valid pipe sequence.
    /// The caller has already consumed the initial '|'.
    fn try_parse_pipe_delimited(&mut self) -> Option<String> {
        let mut key_name = String::new();

        loop {
            match self.input.peek() {
                Some(&'|') => {
                    self.input.next(); // consume closing '|'
                                       // Validate known pipe-delimited keys
                    match key_name.as_str() {
                        "enter" | "tab" | "backspace" | "space" | "escape" => {
                            return Some(key_name)
                        }
                        _ => return None, // Unknown pipe-delimited key
                    }
                }
                Some(&ch) => {
                    self.input.next();
                    key_name.push(ch);
                    if key_name.len() > 10 {
                        return None;
                    }
                }
                None => return None, // End of input without closing '|'
            }
        }
    }

    /// Check if we've reached a terminator for command/search mode.
    /// Returns Some(true) for |enter| (completed), Some(false) for |escape| (cancelled).
    /// Returns None if current position is not a terminator.
    fn check_command_terminator(&mut self) -> Option<bool> {
        match self.input.peek() {
            Some(&'|') => {
                let mut peek_iter = self.input.clone();
                peek_iter.next();

                let mut key_name = String::new();
                loop {
                    match peek_iter.next() {
                        Some('|') => {
                            if key_name == "enter" {
                                // Consume |enter| (7 chars: | e n t e r |)
                                for _ in 0..7 {
                                    self.input.next();
                                }
                                return Some(true);
                            }
                            if key_name == "escape" {
                                // Consume |escape| (8 chars: | e s c a p e |)
                                for _ in 0..8 {
                                    self.input.next();
                                }
                                return Some(false);
                            }
                            return None;
                        }
                        Some(ch) => {
                            key_name.push(ch);
                            if key_name.len() > 10 {
                                return None;
                            }
                        }
                        None => return None,
                    }
                }
            }
            _ => None,
        }
    }

    /// Classify command content and return appropriate token.
    fn classify_command(content: &str, completed: bool) -> Token {
        let trimmed = content.trim();

        if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_digit()) {
            return Token::JumpToLineNumber(trimmed.to_string());
        }

        if trimmed == "h"
            || trimmed.starts_with("h ")
            || trimmed == "help"
            || trimmed.starts_with("help ")
        {
            return Token::HelpPage(completed);
        }

        if trimmed == "w" || trimmed.starts_with("w ") || trimmed.starts_with("w!") {
            return Token::SaveFile(completed);
        }

        Token::Command(completed)
    }

    fn accumulate_digit(&mut self, digit: char) -> u32 {
        self.accumulated_string.push(digit);

        let mut count: u32 = 0;
        for ch in self.accumulated_string.chars() {
            let digit_value = ch.to_digit(10).unwrap_or(0);
            let new_count = count.saturating_mul(10).saturating_add(digit_value);
            if new_count > 999 {
                // Cap at 999 but continue accumulating the string
                count = 999;

                // Consume any remaining digits
                while let Some(&ch) = self.input.peek() {
                    if !ch.is_ascii_digit() {
                        break;
                    }

                    self.input.next();
                    self.accumulated_string.push(ch);
                }

                return count;
            }

            count = new_count;
        }

        count
    }

    /// Maps operator + count to the appropriate token
    fn operator_to_token(operator: Operator, count: u32) -> Token {
        match operator {
            Operator::Delete => Token::DeleteText(i32::try_from(count).unwrap()),
            Operator::Yank => Token::YankPaste,
            Operator::Change => Token::TextManipulationAdvanced,
        }
    }

    /// Maps operator to its character representation (for error messages)
    fn operator_to_char(operator: Operator) -> char {
        match operator {
            Operator::Delete => 'd',
            Operator::Yank => 'y',
            Operator::Change => 'c',
        }
    }

    /// Handles simple command characters that can work with a count.
    /// Returns Some(CommandResult) for commands that can be handled simply,
    /// None for commands that need special state handling.
    fn handle_simple_command(ch: char, count: u32) -> CommandResult {
        let count_i32 = i32::try_from(count).unwrap_or(1);
        match ch {
            'j' | 'k' => CommandResult::Token(Token::MoveVerticalBasic(count_i32)),
            'h' | 'l' => CommandResult::Token(Token::MoveHorizontalBasic(count_i32)),
            'w' | 'W' | 'e' | 'E' | 'b' | 'B' => {
                CommandResult::Token(Token::MoveHorizontalChunk(count_i32))
            }
            'u' | 'U' => CommandResult::Token(Token::UndoRedo),
            '.' => CommandResult::Token(Token::DotRepeat),
            'M' | 'H' | 'L' => CommandResult::Token(Token::JumpToVertical),
            'p' | 'P' | 'Y' => CommandResult::Token(Token::YankPaste),
            'x' | 'J' | 'X' => CommandResult::Token(Token::TextManipulationBasic(count_i32)),
            'D' => CommandResult::Token(Token::DeleteText(count_i32)),
            's' | 'S' | 'C' | '~' => CommandResult::Token(Token::TextManipulationAdvanced),
            'n' | 'N' | ';' | ',' => CommandResult::Token(Token::SearchRepeat),
            '%' => CommandResult::Token(Token::JumpFromContext),
            'f' | 'F' | 't' | 'T' => CommandResult::ConsumeNextOptional(
                Token::JumpToHorizontal,
                Token::Unhandled(ch.to_string()),
            ),
            'r' => CommandResult::ConsumeNextOptional(
                Token::TextManipulationBasic(count_i32),
                Token::Unhandled("r".into()),
            ),
            'm' => CommandResult::ConsumeNextOptional(Token::Marks, Token::Unhandled("m".into())),
            '\'' => CommandResult::ConsumeNextOptional(Token::Marks, Token::Unhandled("'".into())),
            '`' => CommandResult::ConsumeNextOptional(Token::Marks, Token::Unhandled("`".into())),
            _ => CommandResult::NotSimple,
        }
    }

    /// Check if a character is a valid text object specifier
    fn is_text_object_char(ch: char) -> bool {
        matches!(
            ch,
            'w' | 'W' | // word
            '(' | ')' | 'b' | // parentheses
            '{' | '}' | 'B' | // braces
            '[' | ']' | // brackets
            '<' | '>' | // angle brackets
            '\'' | '"' | '`' | // quotes
            't' | // tag
            's' | 'p' // sentence, paragraph
        )
    }

    /// Handle operator pending state - process motion after d, y, or c
    fn handle_operator_motion(&mut self, operator: Operator, count: u32) -> Token {
        let Some(ch) = self.input.next() else {
            // End of input - incomplete operator
            return Token::Unhandled(Self::operator_to_char(operator).to_string());
        };

        // Handle doubled operator (dd, yy, cc) - line operation
        let is_doubled = ch == Self::operator_to_char(operator);

        if is_doubled {
            return Self::operator_to_token(operator, count);
        }

        // Handle motion count (e.g., d3w)
        if ch.is_ascii_digit() && ch != '0' {
            let mut motion_count = ch.to_digit(10).unwrap();
            while let Some(&next_ch) = self.input.peek() {
                if next_ch.is_ascii_digit() {
                    self.input.next();
                    motion_count = motion_count
                        .saturating_mul(10)
                        .saturating_add(next_ch.to_digit(10).unwrap());
                    if motion_count > 999 {
                        motion_count = 999;
                    }
                } else {
                    break;
                }
            }
            let total_count = count.saturating_mul(motion_count);
            return self.handle_operator_with_motion(operator, total_count);
        }

        // Handle text objects (i/a + object)
        if ch == 'i' || ch == 'a' {
            if let Some(&obj_ch) = self.input.peek() {
                if Self::is_text_object_char(obj_ch) {
                    self.input.next(); // consume the object char
                    return Self::operator_to_token(operator, count);
                }
            }
        }

        // Handle regular motions
        self.handle_operator_char_motion(operator, count, ch)
    }

    /// Handle operator with a motion character
    fn handle_operator_char_motion(&mut self, operator: Operator, count: u32, ch: char) -> Token {
        match ch {
            'w' | 'W' | 'e' | 'E' | 'b' | 'B' | '$' | '^' | '0' | 'j' | 'k' | 'h' | 'l' => {
                Self::operator_to_token(operator, count)
            }
            'f' | 'F' | 't' | 'T' => {
                if self.input.next().is_some() {
                    Self::operator_to_token(operator, count)
                } else {
                    Token::Unhandled(format!("{}{ch}", Self::operator_to_char(operator)))
                }
            }
            'g' => {
                if let Some(&next_ch) = self.input.peek() {
                    self.input.next();
                    match next_ch {
                        'g' | 'j' | 'k' | '$' | '^' | '0' | 'e' | 'E' => {
                            Self::operator_to_token(operator, count)
                        }
                        _ => Token::Unhandled(format!(
                            "{}g{next_ch}",
                            Self::operator_to_char(operator)
                        )),
                    }
                } else {
                    Token::Unhandled(format!("{}g", Self::operator_to_char(operator)))
                }
            }
            _ => Token::Unhandled(format!("{}{ch}", Self::operator_to_char(operator))),
        }
    }

    /// Handle operator with accumulated motion count
    fn handle_operator_with_motion(&mut self, operator: Operator, count: u32) -> Token {
        if let Some(ch) = self.input.next() {
            self.handle_operator_char_motion(operator, count, ch)
        } else {
            Token::Unhandled(Self::operator_to_char(operator).to_string())
        }
    }

    /// Handle case operator motion (g~, gu, gU + motion)
    fn handle_case_operator_motion(&mut self, operator: &str, count: u32) -> Token {
        let Some(ch) = self.input.next() else {
            // End of input - incomplete case operator
            return Token::Unhandled(operator.to_string());
        };

        // Handle motion count (e.g., gu3w)
        if ch.is_ascii_digit() && ch != '0' {
            let mut motion_count = ch.to_digit(10).unwrap();
            while let Some(&next_ch) = self.input.peek() {
                if next_ch.is_ascii_digit() {
                    self.input.next();
                    motion_count = motion_count
                        .saturating_mul(10)
                        .saturating_add(next_ch.to_digit(10).unwrap());
                    if motion_count > 999 {
                        motion_count = 999;
                    }
                } else {
                    break;
                }
            }
            return self
                .handle_case_operator_with_motion(operator, count.saturating_mul(motion_count));
        }

        // Handle regular motions
        self.handle_case_operator_char_motion(operator, ch)
    }

    /// Handle case operator with a motion character.
    /// Note: count is not used here because `TextManipulationAdvanced` doesn't carry count info.
    /// The motion itself (not the case operator) determines the range.
    fn handle_case_operator_char_motion(&mut self, operator: &str, ch: char) -> Token {
        match ch {
            'w' | 'W' | 'e' | 'E' | 'b' | 'B' | '$' | '^' | '0' | 'j' | 'k' | 'h' | 'l' => {
                Token::TextManipulationAdvanced
            }
            'f' | 'F' | 't' | 'T' => {
                if self.input.next().is_some() {
                    Token::TextManipulationAdvanced
                } else {
                    Token::Unhandled(format!("{operator}{ch}"))
                }
            }
            'g' => {
                if let Some(&next_ch) = self.input.peek() {
                    self.input.next();
                    match next_ch {
                        'g' | 'j' | 'k' | '$' | '^' | '0' | 'e' | 'E' => {
                            Token::TextManipulationAdvanced
                        }
                        _ => Token::Unhandled(format!("{operator}g{next_ch}")),
                    }
                } else {
                    Token::Unhandled(format!("{operator}g"))
                }
            }
            'i' | 'a' => {
                if let Some(&obj_ch) = self.input.peek() {
                    if Self::is_text_object_char(obj_ch) {
                        self.input.next(); // consume the object char
                        return Token::TextManipulationAdvanced;
                    }
                }
                Token::Unhandled(format!("{operator}{ch}"))
            }
            _ => Token::Unhandled(format!("{operator}{ch}")),
        }
    }

    /// Handle case operator with accumulated motion count
    fn handle_case_operator_with_motion(&mut self, operator: &str, _count: u32) -> Token {
        if let Some(ch) = self.input.next() {
            self.handle_case_operator_char_motion(operator, ch)
        } else {
            Token::Unhandled(operator.to_string())
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn next_token(&mut self) -> Option<Token> {
        let mode_content = match &mut self.state {
            State::CommandMode { content } => Some((0, std::mem::take(content))),
            State::SearchMode { content } => Some((1, std::mem::take(content))),
            State::ReplaceMode { content } => Some((2, std::mem::take(content))),
            _ => None,
        };

        if let Some((mode_type, mut content)) = mode_content {
            if mode_type == 2 {
                loop {
                    // Check for |escape| terminator
                    if let Some(&'|') = self.input.peek() {
                        let mut peek_iter = self.input.clone();
                        peek_iter.next(); // skip '|'
                        let chars: String = peek_iter.take(7).collect();
                        if chars == "escape|" {
                            // Consume |escape| from real iterator
                            for _ in 0..8 {
                                self.input.next();
                            }
                            self.state = State::None;
                            return Some(Token::TextManipulationAdvanced);
                        }
                    }

                    if let Some(ch) = self.input.next() {
                        content.push(ch);
                    } else {
                        // End of input without |escape| - treat as incomplete
                        self.state = State::None;
                        return Some(Token::TextManipulationAdvanced);
                    }
                }
            }

            // Continue accumulating content until terminator (CommandMode or SearchMode)
            loop {
                // Check for terminator first
                if let Some(completed) = self.check_command_terminator() {
                    self.state = State::None;
                    if mode_type == 0 {
                        return Some(Self::classify_command(&content, completed));
                    }
                    return Some(Token::CommandSearch(completed));
                }

                // Handle pipe-delimited keys within command/search (like |space|)
                if let Some(&'|') = self.input.peek() {
                    self.input.next(); // consume '|'
                    if let Some(key) = self.try_parse_pipe_delimited() {
                        match key.as_str() {
                            "space" => content.push(' '),
                            "tab" => content.push('\t'),
                            "backspace" => {
                                content.pop();
                            }
                            _ => {} // "enter" should have been caught by check_command_terminator
                        }
                        continue;
                    }
                    // Not a valid pipe sequence, treat '|' as literal
                    content.push('|');
                    continue;
                }

                if let Some(ch) = self.input.next() {
                    content.push(ch);
                } else {
                    // End of input without terminator - treat as incomplete
                    self.state = State::None;
                    if mode_type == 0 {
                        return Some(Token::Command(false));
                    }
                    return Some(Token::CommandSearch(false));
                }
            }
        }

        match self.state {
            State::CommandMode { .. } | State::SearchMode { .. } | State::ReplaceMode { .. } => {
                unreachable!("Already handled above")
            }
            State::CaseOperatorPending {
                ref operator,
                count,
            } => {
                let op = operator.clone();
                self.state = State::None;
                Some(self.handle_case_operator_motion(&op, count))
            }
            State::OperatorPending { operator, count } => {
                self.state = State::None;
                Some(self.handle_operator_motion(operator, count))
            }
            State::AccumulatingCount(count) => {
                if let Some(&ch) = self.input.peek() {
                    if ch.is_ascii_digit() {
                        // Continue accumulating
                        self.input.next();
                        let new_count = self.accumulate_digit(ch);
                        self.state = State::AccumulatingCount(new_count);

                        // Continue accumulating
                        self.next_token()
                    } else {
                        // Non-digit encountered, process based on what it is
                        self.state = State::None;
                        let accumulated = self.accumulated_string.clone();
                        self.accumulated_string.clear();

                        // Try simple command handling first
                        match Self::handle_simple_command(ch, count) {
                            CommandResult::Token(token) => {
                                self.input.next();
                                return Some(token);
                            }
                            CommandResult::ConsumeNextOptional(success, failure) => {
                                self.input.next(); // consume command char
                                return if self.input.next().is_some() {
                                    Some(success)
                                } else {
                                    Some(failure)
                                };
                            }
                            CommandResult::NotSimple => {} // Fall through to special handling
                        }

                        // Special commands that need state transitions or complex handling
                        match ch {
                            'G' => {
                                self.input.next();
                                Some(Token::JumpToLineNumber(accumulated))
                            }
                            'g' => {
                                self.input.next();
                                match self.input.next() {
                                    Some('j' | 'k') => Some(Token::MoveVerticalBasic(
                                        i32::try_from(count).unwrap(),
                                    )),
                                    Some('g') => Some(Token::JumpToLineNumber(accumulated)),
                                    Some('J') => Some(Token::TextManipulationBasic(
                                        i32::try_from(count).unwrap(),
                                    )),
                                    Some('~') => {
                                        self.state = State::CaseOperatorPending {
                                            operator: "g~".to_string(),
                                            count,
                                        };
                                        self.next_token()
                                    }
                                    Some('u') => {
                                        self.state = State::CaseOperatorPending {
                                            operator: "gu".to_string(),
                                            count,
                                        };
                                        self.next_token()
                                    }
                                    Some('U') => {
                                        self.state = State::CaseOperatorPending {
                                            operator: "gU".to_string(),
                                            count,
                                        };
                                        self.next_token()
                                    }
                                    Some(ch) => Some(Token::Unhandled(format!("g{ch}"))),
                                    None => Some(Token::Unhandled("g".into())),
                                }
                            }
                            'z' => {
                                self.input.next();
                                match self.input.next() {
                                    Some('z' | 't' | 'b') => Some(Token::CameraMovement),
                                    Some(ch) => Some(Token::Unhandled(format!("z{ch}"))),
                                    None => Some(Token::Unhandled("z".into())),
                                }
                            }
                            'f' | 'F' | 't' | 'T' => {
                                self.input.next();
                                if self.input.next().is_some() {
                                    Some(Token::JumpToHorizontal)
                                } else {
                                    Some(Token::Unhandled(ch.to_string()))
                                }
                            }
                            'r' => {
                                self.input.next();
                                if self.input.next().is_some() {
                                    Some(Token::TextManipulationBasic(
                                        i32::try_from(count).unwrap(),
                                    ))
                                } else {
                                    Some(Token::Unhandled("r".into()))
                                }
                            }
                            'd' => {
                                self.input.next();
                                self.state = State::OperatorPending {
                                    operator: Operator::Delete,
                                    count,
                                };
                                self.next_token()
                            }
                            'y' => {
                                self.input.next();
                                self.state = State::OperatorPending {
                                    operator: Operator::Yank,
                                    count,
                                };
                                self.next_token()
                            }
                            'c' => {
                                self.input.next();
                                self.state = State::OperatorPending {
                                    operator: Operator::Change,
                                    count,
                                };
                                self.next_token()
                            }
                            '<' => {
                                self.input.next();
                                if let Some(ctrl_char) = self.try_parse_control_sequence() {
                                    Some(self.handle_control_sequence(ctrl_char, count))
                                } else {
                                    Some(Token::Unhandled(accumulated))
                                }
                            }
                            _ => Some(Token::Unhandled(accumulated)),
                        }
                    }
                } else {
                    self.state = State::None;
                    let accumulated = self.accumulated_string.clone();
                    self.accumulated_string.clear();
                    Some(Token::Unhandled(accumulated))
                }
            }
            State::None => {
                let ch = self.input.next()?;

                if ch.is_ascii_digit() && ch != '0' {
                    let count = self.accumulate_digit(ch);
                    self.state = State::AccumulatingCount(count);
                    return self.next_token();
                }

                match Self::handle_simple_command(ch, 1) {
                    CommandResult::Token(token) => return Some(token),
                    CommandResult::ConsumeNextOptional(success, failure) => {
                        return if self.input.next().is_some() {
                            Some(success)
                        } else {
                            Some(failure)
                        };
                    }
                    CommandResult::NotSimple => {} // Fall through to special handling
                }

                // Special commands that need state transitions or complex handling
                match ch {
                    '0' => Some(Token::Unhandled("0".into())),
                    'G' => Some(Token::JumpToLineNumber(String::new())),
                    'g' => match self.input.next() {
                        Some('j' | 'k') => Some(Token::MoveVerticalBasic(1)),
                        Some('g') => Some(Token::JumpToLineNumber(String::new())),
                        Some('J') => Some(Token::TextManipulationBasic(1)),
                        Some('~') => {
                            self.state = State::CaseOperatorPending {
                                operator: "g~".to_string(),
                                count: 1,
                            };
                            self.next_token()
                        }
                        Some('u') => {
                            self.state = State::CaseOperatorPending {
                                operator: "gu".to_string(),
                                count: 1,
                            };
                            self.next_token()
                        }
                        Some('U') => {
                            self.state = State::CaseOperatorPending {
                                operator: "gU".to_string(),
                                count: 1,
                            };
                            self.next_token()
                        }
                        Some(ch) => Some(Token::Unhandled(format!("g{ch}"))),
                        None => Some(Token::Unhandled("g".into())),
                    },
                    'R' => {
                        self.state = State::ReplaceMode {
                            content: String::new(),
                        };
                        self.next_token()
                    }
                    'z' => match self.input.next() {
                        Some('z' | 't' | 'b') => Some(Token::CameraMovement),
                        Some(ch) => Some(Token::Unhandled(format!("z{ch}"))),
                        None => Some(Token::Unhandled("z".into())),
                    },
                    'd' => {
                        self.state = State::OperatorPending {
                            operator: Operator::Delete,
                            count: 1,
                        };
                        self.next_token()
                    }
                    'y' => {
                        self.state = State::OperatorPending {
                            operator: Operator::Yank,
                            count: 1,
                        };
                        self.next_token()
                    }
                    'c' => {
                        self.state = State::OperatorPending {
                            operator: Operator::Change,
                            count: 1,
                        };
                        self.next_token()
                    }
                    '<' => {
                        if let Some(ctrl_char) = self.try_parse_control_sequence() {
                            Some(self.handle_control_sequence(ctrl_char, 1))
                        } else {
                            Some(Token::Unhandled("<".into()))
                        }
                    }
                    '|' => {
                        if let Some(key) = self.try_parse_pipe_delimited() {
                            match key.as_str() {
                                "escape" => Some(Token::Unhandled("|escape|".into())),
                                _ => Some(Token::Unhandled(format!("|{key}|"))),
                            }
                        } else {
                            Some(Token::Unhandled("|".into()))
                        }
                    }
                    ':' => {
                        self.state = State::CommandMode {
                            content: String::new(),
                        };
                        self.next_token()
                    }
                    '/' | '?' => {
                        self.state = State::SearchMode {
                            content: String::new(),
                        };
                        self.next_token()
                    }
                    // Intentionally unhandled commands fall through here:
                    // - Insert mode: o, O, i, I, a, A (enter insert mode - handled by game)
                    // - Visual mode: v, V (future consideration per SPEC.md)
                    // - Macros: q, @ (future consideration per SPEC.md)
                    // See module-level documentation for full details.
                    _ => Some(Token::Unhandled(ch.into())),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_prefix_basic() {
        let mut lexer = Lexer::new("5j");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(5))
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_numeric_prefix_large() {
        let mut lexer = Lexer::new("123k");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(123))
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_numeric_prefix_cap() {
        let mut lexer = Lexer::new("999j");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(999))
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_numeric_prefix_overflow() {
        let mut lexer = Lexer::new("1234567j");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(999))
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_no_numeric_prefix() {
        let mut lexer = Lexer::new("j");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(1))
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_standalone_digits() {
        let mut lexer = Lexer::new("123");
        assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "123"));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_leading_zero() {
        let mut lexer = Lexer::new("0j");
        assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "0"));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(1))
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_multiple_commands() {
        let mut lexer = Lexer::new("2j3k");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(2))
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(3))
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_mixed_input() {
        let mut lexer = Lexer::new("j5ka");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(1))
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(5))
        ));
        assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "a"));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_standalone_digits_overflow() {
        let mut lexer = Lexer::new("999999");
        assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "999999"));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_mixed_digits_and_commands() {
        let mut lexer = Lexer::new("12x34j");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationBasic(12))
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(34))
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_move_horizontal_chunk_and_move_vertical() {
        let mut lexer = Lexer::new("2w3kbj");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveHorizontalChunk(2))
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(3))
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveHorizontalChunk(1))
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(1))
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn no_input_as_none() {
        let mut lexer = Lexer::new("");
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_horizontal_basic() {
        let mut lexer = Lexer::new("h5l");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveHorizontalBasic(1))
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveHorizontalBasic(5))
        ));
    }

    #[test]
    fn test_horizontal_chunk() {
        let mut lexer = Lexer::new("wWeEbB");
        for _ in 0..6 {
            assert!(matches!(
                lexer.next_token(),
                Some(Token::MoveHorizontalChunk(1))
            ));
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
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationBasic(1))
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationBasic(5))
        ));
    }

    #[test]
    fn test_text_manipulation_basic_j() {
        let mut lexer = Lexer::new("J3J");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationBasic(1))
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationBasic(3))
        ));
    }

    #[test]
    fn test_jump_to_line_g() {
        let mut lexer = Lexer::new("G10G");
        assert!(matches!(lexer.next_token(), Some(Token::JumpToLineNumber(ref s)) if s.is_empty()));
        assert!(matches!(lexer.next_token(), Some(Token::JumpToLineNumber(ref s)) if s == "10"));
    }

    #[test]
    fn test_control_vertical_chunk() {
        let mut lexer = Lexer::new("<C-U><C-D>5<C-U>");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalChunk(1))
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalChunk(1))
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalChunk(5))
        ));
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

    #[test]
    fn test_invalid_control_sequence() {
        let mut lexer = Lexer::new("<C-X>");
        assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "<C-X>"));
    }

    #[test]
    fn test_incomplete_control_sequence() {
        let mut lexer = Lexer::new("<C-");
        assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "<"));
    }

    #[test]
    fn test_g_vertical_movement() {
        let mut lexer = Lexer::new("gj5gk");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(1))
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(5))
        ));
    }

    #[test]
    fn test_gg_jump() {
        let mut lexer = Lexer::new("gg10gg");
        assert!(matches!(lexer.next_token(), Some(Token::JumpToLineNumber(ref s)) if s.is_empty()));
        assert!(matches!(lexer.next_token(), Some(Token::JumpToLineNumber(ref s)) if s == "10"));
    }

    #[test]
    fn test_gj_text_manipulation() {
        let mut lexer = Lexer::new("gJ3gJ");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationBasic(1))
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationBasic(3))
        ));
    }

    #[test]
    fn test_z_camera_movement() {
        let mut lexer = Lexer::new("ztzb");
        assert!(matches!(lexer.next_token(), Some(Token::CameraMovement)));
        assert!(matches!(lexer.next_token(), Some(Token::CameraMovement)));
    }

    #[test]
    fn test_zz_camera_movement() {
        let mut lexer = Lexer::new("zz");
        assert!(matches!(lexer.next_token(), Some(Token::CameraMovement)));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_zz_multiple() {
        // Two zz commands
        let mut lexer = Lexer::new("zzzz");
        assert!(matches!(lexer.next_token(), Some(Token::CameraMovement)));
        assert!(matches!(lexer.next_token(), Some(Token::CameraMovement)));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_zz_mixed() {
        let mut lexer = Lexer::new("zzztzb");
        assert!(matches!(lexer.next_token(), Some(Token::CameraMovement)));
        assert!(matches!(lexer.next_token(), Some(Token::CameraMovement)));
        assert!(matches!(lexer.next_token(), Some(Token::CameraMovement)));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_unrecognized_g_prefix() {
        let mut lexer = Lexer::new("gx");
        assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "gx"));
    }

    #[test]
    fn test_unrecognized_z_prefix() {
        let mut lexer = Lexer::new("zx");
        assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "zx"));
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
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationBasic(1))
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationBasic(5))
        ));
    }

    #[test]
    fn test_jump_horizontal_incomplete() {
        let mut lexer = Lexer::new("f");
        assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "f"));
    }

    #[test]
    fn test_replace_char_incomplete() {
        let mut lexer = Lexer::new("r");
        assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "r"));
    }

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
        let mut lexer = Lexer::new("ywy$");
        assert!(matches!(lexer.next_token(), Some(Token::YankPaste)));
        assert!(matches!(lexer.next_token(), Some(Token::YankPaste)));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_change() {
        let mut lexer = Lexer::new("cccwc$");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
    }

    #[test]
    fn test_change_aliases() {
        let mut lexer = Lexer::new("sSC");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
    }

    #[test]
    fn test_text_objects() {
        let mut lexer = Lexer::new("ciwcawci)ca}");
        for _ in 0..4 {
            assert!(matches!(
                lexer.next_token(),
                Some(Token::TextManipulationAdvanced)
            ));
        }
    }

    #[test]
    fn test_delete_text_objects() {
        let mut lexer = Lexer::new("diwdawdi)da}");
        for _ in 0..4 {
            assert!(matches!(lexer.next_token(), Some(Token::DeleteText(1))));
        }
    }

    #[test]
    fn test_yank_text_objects() {
        let mut lexer = Lexer::new("yiwyawyi)ya}");
        for _ in 0..4 {
            assert!(matches!(lexer.next_token(), Some(Token::YankPaste)));
        }
    }

    #[test]
    fn test_y_uppercase() {
        let mut lexer = Lexer::new("Y");
        assert!(matches!(lexer.next_token(), Some(Token::YankPaste)));
    }

    #[test]
    fn test_incomplete_d() {
        let mut lexer = Lexer::new("d");
        assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "d"));
    }

    #[test]
    fn test_incomplete_y() {
        let mut lexer = Lexer::new("y");
        assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "y"));
    }

    #[test]
    fn test_incomplete_c() {
        let mut lexer = Lexer::new("c");
        assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "c"));
    }

    #[test]
    fn test_delete_with_find_motion() {
        let mut lexer = Lexer::new("dfxdta");
        assert!(matches!(lexer.next_token(), Some(Token::DeleteText(1))));
        assert!(matches!(lexer.next_token(), Some(Token::DeleteText(1))));
    }

    #[test]
    fn test_operator_with_g_motion() {
        let mut lexer = Lexer::new("dggyggygg");
        assert!(matches!(lexer.next_token(), Some(Token::DeleteText(1))));
        assert!(matches!(lexer.next_token(), Some(Token::YankPaste)));
        assert!(matches!(lexer.next_token(), Some(Token::YankPaste)));
    }

    #[test]
    fn test_search_completed() {
        let mut lexer = Lexer::new("/test|enter|");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::CommandSearch(true))
        ));
    }

    #[test]
    fn test_search_cancelled() {
        let mut lexer = Lexer::new("/test|escape|");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::CommandSearch(false))
        ));
    }

    #[test]
    fn test_search_backward() {
        let mut lexer = Lexer::new("?pattern|enter|");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::CommandSearch(true))
        ));
    }

    #[test]
    fn test_command_line_number() {
        let mut lexer = Lexer::new(":42|enter|");
        assert!(matches!(lexer.next_token(), Some(Token::JumpToLineNumber(ref s)) if s == "42"));
    }

    #[test]
    fn test_help_page() {
        let mut lexer = Lexer::new(":h test|enter|:help topic|escape|");
        assert!(matches!(lexer.next_token(), Some(Token::HelpPage(true))));
        assert!(matches!(lexer.next_token(), Some(Token::HelpPage(false))));
    }

    #[test]
    fn test_save_file() {
        let mut lexer = Lexer::new(":w|enter|:w|escape|");
        assert!(matches!(lexer.next_token(), Some(Token::SaveFile(true))));
        assert!(matches!(lexer.next_token(), Some(Token::SaveFile(false))));
    }

    #[test]
    fn test_generic_command() {
        let mut lexer = Lexer::new(":Vimscape|enter|:q|escape|");
        assert!(matches!(lexer.next_token(), Some(Token::Command(true))));
        assert!(matches!(lexer.next_token(), Some(Token::Command(false))));
    }

    #[test]
    fn test_command_with_space() {
        let mut lexer = Lexer::new(":Vimscape|space|toggle|enter|");
        assert!(matches!(lexer.next_token(), Some(Token::Command(true))));
    }

    #[test]
    fn test_replace_mode() {
        let mut lexer = Lexer::new("Rtest|escape|");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
    }

    #[test]
    fn test_replace_mode_empty() {
        let mut lexer = Lexer::new("R|escape|");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
    }

    #[test]
    fn test_case_toggle() {
        let mut lexer = Lexer::new("g~w");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
    }

    #[test]
    fn test_case_lower() {
        let mut lexer = Lexer::new("guw");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
    }

    #[test]
    fn test_case_upper() {
        let mut lexer = Lexer::new("gUw");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
    }

    #[test]
    fn test_case_with_count() {
        let mut lexer = Lexer::new("gu3w");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
    }

    #[test]
    fn test_case_with_prefix_count() {
        let mut lexer = Lexer::new("3guw");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
    }

    #[test]
    fn test_case_operators_various_motions() {
        let mut lexer = Lexer::new("g~$guegUb");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
    }

    #[test]
    fn test_incomplete_case_operator() {
        let mut lexer = Lexer::new("gu");
        assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "gu"));
    }

    #[test]
    fn test_jump_from_context() {
        let mut lexer = Lexer::new("%");
        assert!(matches!(lexer.next_token(), Some(Token::JumpFromContext)));
    }

    #[test]
    fn test_escape_outside_command() {
        let mut lexer = Lexer::new("|escape|");
        assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "|escape|"));
    }

    #[test]
    fn test_mixed_complex_sequence() {
        let mut lexer = Lexer::new("5j/test|enter|dd3gkzz:w|enter|");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(5))
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::CommandSearch(true))
        ));
        assert!(matches!(lexer.next_token(), Some(Token::DeleteText(1))));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(3))
        ));
        assert!(matches!(lexer.next_token(), Some(Token::CameraMovement)));
        assert!(matches!(lexer.next_token(), Some(Token::SaveFile(true))));
    }

    #[test]
    fn test_percent_with_other_commands() {
        let mut lexer = Lexer::new("j%k");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(1))
        ));
        assert!(matches!(lexer.next_token(), Some(Token::JumpFromContext)));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(1))
        ));
    }

    #[test]
    fn test_delete_to_end_of_line() {
        let mut lexer = Lexer::new("D3D");
        assert!(matches!(lexer.next_token(), Some(Token::DeleteText(1))));
        assert!(matches!(lexer.next_token(), Some(Token::DeleteText(3))));
    }

    #[test]
    fn test_delete_char_before() {
        let mut lexer = Lexer::new("X5X");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationBasic(1))
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationBasic(5))
        ));
    }

    #[test]
    fn test_toggle_case_single() {
        let mut lexer = Lexer::new("~");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
    }

    #[test]
    fn test_toggle_case_multiple() {
        let mut lexer = Lexer::new("~~");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
    }

    #[test]
    fn test_search_repeat_n() {
        let mut lexer = Lexer::new("n");
        assert!(matches!(lexer.next_token(), Some(Token::SearchRepeat)));
    }

    #[test]
    fn test_search_repeat_n_upper() {
        let mut lexer = Lexer::new("N");
        assert!(matches!(lexer.next_token(), Some(Token::SearchRepeat)));
    }

    #[test]
    fn test_find_repeat_semicolon() {
        let mut lexer = Lexer::new(";");
        assert!(matches!(lexer.next_token(), Some(Token::SearchRepeat)));
    }

    #[test]
    fn test_find_repeat_comma() {
        let mut lexer = Lexer::new(",");
        assert!(matches!(lexer.next_token(), Some(Token::SearchRepeat)));
    }

    #[test]
    fn test_search_repeat_sequence() {
        let mut lexer = Lexer::new("nN;,");
        assert!(matches!(lexer.next_token(), Some(Token::SearchRepeat)));
        assert!(matches!(lexer.next_token(), Some(Token::SearchRepeat)));
        assert!(matches!(lexer.next_token(), Some(Token::SearchRepeat)));
        assert!(matches!(lexer.next_token(), Some(Token::SearchRepeat)));
    }

    #[test]
    fn test_mark_set() {
        let mut lexer = Lexer::new("mamzmA");
        assert!(matches!(lexer.next_token(), Some(Token::Marks)));
        assert!(matches!(lexer.next_token(), Some(Token::Marks)));
        assert!(matches!(lexer.next_token(), Some(Token::Marks)));
    }

    #[test]
    fn test_mark_jump_line() {
        let mut lexer = Lexer::new("'a'z");
        assert!(matches!(lexer.next_token(), Some(Token::Marks)));
        assert!(matches!(lexer.next_token(), Some(Token::Marks)));
    }

    #[test]
    fn test_mark_jump_position() {
        let mut lexer = Lexer::new("`a`z");
        assert!(matches!(lexer.next_token(), Some(Token::Marks)));
        assert!(matches!(lexer.next_token(), Some(Token::Marks)));
    }

    #[test]
    fn test_mark_incomplete() {
        let mut lexer = Lexer::new("m");
        assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "m"));
    }

    #[test]
    fn test_mark_quote_incomplete() {
        let mut lexer = Lexer::new("'");
        assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "'"));
    }

    #[test]
    fn test_mark_backtick_incomplete() {
        let mut lexer = Lexer::new("`");
        assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "`"));
    }

    // =========================================================================
    // Raw Typed Input Tests
    // =========================================================================
    // These tests use raw typed input (as received from vim.on_key's second
    // argument), without any Neovim internal key transformations.

    #[test]
    fn test_x_raw() {
        let mut lexer = Lexer::new("x");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationBasic(1))
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_x_with_count_raw() {
        let mut lexer = Lexer::new("5x");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationBasic(5))
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_s_raw() {
        let mut lexer = Lexer::new("s");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_s_uppercase_raw() {
        let mut lexer = Lexer::new("S");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_x_and_s_mixed_raw() {
        let mut lexer = Lexer::new("xs");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationBasic(1))
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_paste_raw() {
        let mut lexer = Lexer::new("p");
        assert!(matches!(lexer.next_token(), Some(Token::YankPaste)));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_paste_uppercase_raw() {
        let mut lexer = Lexer::new("P");
        assert!(matches!(lexer.next_token(), Some(Token::YankPaste)));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_paste_with_count_raw() {
        let mut lexer = Lexer::new("3p");
        assert!(matches!(lexer.next_token(), Some(Token::YankPaste)));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_yy_raw() {
        let mut lexer = Lexer::new("yy");
        assert!(matches!(lexer.next_token(), Some(Token::YankPaste)));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_yy_multiple_raw() {
        // Two yy commands
        let mut lexer = Lexer::new("yyyy");
        assert!(matches!(lexer.next_token(), Some(Token::YankPaste)));
        assert!(matches!(lexer.next_token(), Some(Token::YankPaste)));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_yy_with_count_raw() {
        let mut lexer = Lexer::new("3yy");
        assert!(matches!(lexer.next_token(), Some(Token::YankPaste)));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_delete_word_raw() {
        let mut lexer = Lexer::new("dw");
        assert!(matches!(lexer.next_token(), Some(Token::DeleteText(1))));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_delete_end_raw() {
        let mut lexer = Lexer::new("d$");
        assert!(matches!(lexer.next_token(), Some(Token::DeleteText(1))));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_delete_various_motions_raw() {
        let mut lexer = Lexer::new("dWdedEdBdB");
        assert!(matches!(lexer.next_token(), Some(Token::DeleteText(1))));
        assert!(matches!(lexer.next_token(), Some(Token::DeleteText(1))));
        assert!(matches!(lexer.next_token(), Some(Token::DeleteText(1))));
        assert!(matches!(lexer.next_token(), Some(Token::DeleteText(1))));
        assert!(matches!(lexer.next_token(), Some(Token::DeleteText(1))));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_change_word_raw() {
        let mut lexer = Lexer::new("cw");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_change_end_raw() {
        let mut lexer = Lexer::new("c$");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_case_operator_raw() {
        let mut lexer = Lexer::new("guwgu$");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_change_inner_word_raw() {
        let mut lexer = Lexer::new("ciw");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_change_around_word_raw() {
        let mut lexer = Lexer::new("caw");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_delete_inner_word_raw() {
        let mut lexer = Lexer::new("diw");
        assert!(matches!(lexer.next_token(), Some(Token::DeleteText(1))));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_change_inner_paren_raw() {
        let mut lexer = Lexer::new("ci)");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_change_inner_brace_raw() {
        let mut lexer = Lexer::new("ci{");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_change_inner_bracket_raw() {
        let mut lexer = Lexer::new("ci[");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_multiple_text_objects_raw() {
        let mut lexer = Lexer::new("ciwcaw");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::TextManipulationAdvanced)
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_percent_raw() {
        // With typed arg, % arrives as raw % (not matchit expansion)
        let mut lexer = Lexer::new("%");
        assert!(matches!(lexer.next_token(), Some(Token::JumpFromContext)));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_percent_with_following_command_raw() {
        let mut lexer = Lexer::new("%j");
        assert!(matches!(lexer.next_token(), Some(Token::JumpFromContext)));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(1))
        ));
        assert!(lexer.next_token().is_none());
    }
}

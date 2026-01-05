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

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    state: State,
    accumulated_string: String,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let chars = input.chars().peekable();
        Self {
            input: chars,
            state: State::None,
            accumulated_string: String::new(),
        }
    }

    /// Try to parse a control sequence like `<C-X>`.
    /// Returns Some(char) with the control character (e.g., 'U', 'D', 'F', etc.) if valid.
    /// Returns None if not a valid control sequence.
    /// The caller has already consumed the initial '<'.
    fn try_parse_control_sequence(&mut self) -> Option<char> {
        // We need to peek ahead to check for C-X> pattern
        // Expected: C-X> where X is a single character

        // Check for 'C'
        if self.input.peek() != Some(&'C') {
            return None;
        }
        self.input.next(); // consume 'C'

        // Check for '-'
        if self.input.peek() != Some(&'-') {
            return None;
        }
        self.input.next(); // consume '-'

        // Get the control character
        let ctrl_char = self.input.next()?;

        // Check for closing '>'
        if self.input.peek() != Some(&'>') {
            return None;
        }
        self.input.next(); // consume '>'

        Some(ctrl_char)
    }

    /// Handle a control sequence and return the appropriate token.
    /// `count` is the numeric prefix (default 1).
    fn handle_control_sequence(&mut self, ctrl_char: char, count: u32) -> Token {
        match ctrl_char {
            // <C-U>, <C-D> -> MoveVerticalChunk(n)
            'U' | 'D' => Token::MoveVerticalChunk(i32::try_from(count).unwrap()),

            // <C-F>, <C-B> -> JumpToVertical
            'F' | 'B' => Token::JumpToVertical,

            // <C-E>, <C-Y> -> CameraMovement
            'E' | 'Y' => Token::CameraMovement,

            // <C-R> -> UndoRedo
            'R' => Token::UndoRedo,

            // <C-H>, <C-J>, <C-K>, <C-L> -> WindowManagement
            'H' | 'J' | 'K' | 'L' => Token::WindowManagement,

            // <C-W> -> WindowManagement (consumes next character)
            'W' => {
                // Consume the next character as part of the window command
                // e.g., <C-W>s, <C-W>v, etc.
                let _ = self.input.next();
                Token::WindowManagement
            }

            // Unrecognized control sequence
            _ => Token::Unhandled(format!("<C-{ctrl_char}>")),
        }
    }

    /// Try to parse a pipe-delimited special key like `|enter|`, `|tab|`, etc.
    /// Returns `Some(key_name)` if valid (e.g., "enter", "tab", "`backspace`", "space").
    /// Returns None if not a valid pipe sequence.
    /// The caller has already consumed the initial '|'.
    fn try_parse_pipe_delimited(&mut self) -> Option<String> {
        let mut key_name = String::new();

        // Accumulate characters until we hit another '|' or end of input
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
                    // Limit length to prevent infinite accumulation
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
                // Clone iterator to peek ahead without consuming
                let mut peek_iter = self.input.clone();
                peek_iter.next(); // skip '|'

                // Check for "enter|" or "escape|"
                let mut key_name = String::new();
                loop {
                    match peek_iter.next() {
                        Some('|') => {
                            if key_name == "enter" {
                                // Consume the |enter| from the real iterator
                                self.input.next(); // '|'
                                for _ in 0..5 {
                                    self.input.next(); // "enter"
                                }
                                self.input.next(); // '|'
                                return Some(true);
                            }
                            if key_name == "escape" {
                                // Consume the |escape| from the real iterator
                                self.input.next(); // '|'
                                for _ in 0..6 {
                                    self.input.next(); // "escape"
                                }
                                self.input.next(); // '|'
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

        // Check for line number: all digits
        if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_digit()) {
            return Token::JumpToLineNumber(trimmed.to_string());
        }

        // Check for help: starts with 'h' or 'help'
        if trimmed == "h"
            || trimmed.starts_with("h ")
            || trimmed == "help"
            || trimmed.starts_with("help ")
        {
            return Token::HelpPage(completed);
        }

        // Check for save: 'w' or 'w ' (write)
        if trimmed == "w" || trimmed.starts_with("w ") || trimmed.starts_with("w!") {
            return Token::SaveFile(completed);
        }

        // Generic command
        Token::Command(completed)
    }

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
            let op_char = match operator {
                Operator::Delete => 'd',
                Operator::Yank => 'y',
                Operator::Change => 'c',
            };
            return Token::Unhandled(op_char.to_string());
        };

        // Handle doubled operator (dd, yy, cc) - line operation
        let is_doubled = match operator {
            Operator::Delete => ch == 'd',
            Operator::Yank => ch == 'y',
            Operator::Change => ch == 'c',
        };

        if is_doubled {
            return match operator {
                Operator::Delete => Token::DeleteText(i32::try_from(count).unwrap()),
                Operator::Yank => Token::YankPaste,
                Operator::Change => Token::TextManipulationAdvanced,
            };
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
            // Now get the actual motion
            let total_count = count.saturating_mul(motion_count);
            return self.handle_operator_with_motion(operator, total_count);
        }

        // Handle text objects (i/a + object)
        if ch == 'i' || ch == 'a' {
            if let Some(&obj_ch) = self.input.peek() {
                if Self::is_text_object_char(obj_ch) {
                    self.input.next(); // consume the object char
                    return match operator {
                        Operator::Delete => Token::DeleteText(i32::try_from(count).unwrap()),
                        Operator::Yank => Token::YankPaste,
                        Operator::Change => Token::TextManipulationAdvanced,
                    };
                }
            }
        }

        // Handle regular motions
        self.handle_operator_char_motion(operator, count, ch)
    }

    /// Handle operator with a motion character
    fn handle_operator_char_motion(&mut self, operator: Operator, count: u32, ch: char) -> Token {
        match ch {
            // Word/chunk motions, line position motions, basic movements
            'w' | 'W' | 'e' | 'E' | 'b' | 'B' | '$' | '^' | '0' | 'j' | 'k' | 'h' | 'l' => {
                match operator {
                    Operator::Delete => Token::DeleteText(i32::try_from(count).unwrap()),
                    Operator::Yank => Token::YankPaste,
                    Operator::Change => Token::TextManipulationAdvanced,
                }
            }
            // Find char motions (consume target char)
            'f' | 'F' | 't' | 'T' => {
                // Consume the target character
                if self.input.next().is_some() {
                    match operator {
                        Operator::Delete => Token::DeleteText(i32::try_from(count).unwrap()),
                        Operator::Yank => Token::YankPaste,
                        Operator::Change => Token::TextManipulationAdvanced,
                    }
                } else {
                    // Incomplete find motion
                    let op_char = match operator {
                        Operator::Delete => 'd',
                        Operator::Yank => 'y',
                        Operator::Change => 'c',
                    };
                    Token::Unhandled(format!("{op_char}{ch}"))
                }
            }
            // g-prefix motions (gg, gj, gk, g$, etc.)
            'g' => {
                if let Some(&next_ch) = self.input.peek() {
                    self.input.next();
                    match next_ch {
                        'g' | 'j' | 'k' | '$' | '^' | '0' | 'e' | 'E' => match operator {
                            Operator::Delete => Token::DeleteText(i32::try_from(count).unwrap()),
                            Operator::Yank => Token::YankPaste,
                            Operator::Change => Token::TextManipulationAdvanced,
                        },
                        _ => {
                            let op_char = match operator {
                                Operator::Delete => 'd',
                                Operator::Yank => 'y',
                                Operator::Change => 'c',
                            };
                            Token::Unhandled(format!("{op_char}g{next_ch}"))
                        }
                    }
                } else {
                    let op_char = match operator {
                        Operator::Delete => 'd',
                        Operator::Yank => 'y',
                        Operator::Change => 'c',
                    };
                    Token::Unhandled(format!("{op_char}g"))
                }
            }
            // Unrecognized motion
            _ => {
                let op_char = match operator {
                    Operator::Delete => 'd',
                    Operator::Yank => 'y',
                    Operator::Change => 'c',
                };
                Token::Unhandled(format!("{op_char}{ch}"))
            }
        }
    }

    /// Handle operator with accumulated motion count
    fn handle_operator_with_motion(&mut self, operator: Operator, count: u32) -> Token {
        if let Some(ch) = self.input.next() {
            self.handle_operator_char_motion(operator, count, ch)
        } else {
            let op_char = match operator {
                Operator::Delete => 'd',
                Operator::Yank => 'y',
                Operator::Change => 'c',
            };
            Token::Unhandled(op_char.to_string())
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
            // Now get the actual motion
            return self
                .handle_case_operator_with_motion(operator, count.saturating_mul(motion_count));
        }

        // Handle regular motions
        self.handle_case_operator_char_motion(operator, count, ch)
    }

    /// Handle case operator with a motion character
    fn handle_case_operator_char_motion(&mut self, operator: &str, _count: u32, ch: char) -> Token {
        match ch {
            // Word/chunk motions, line position motions, basic movements
            'w' | 'W' | 'e' | 'E' | 'b' | 'B' | '$' | '^' | '0' | 'j' | 'k' | 'h' | 'l' => {
                Token::TextManipulationAdvanced
            }
            // Find char motions (consume target char)
            'f' | 'F' | 't' | 'T' => {
                // Consume the target character
                if self.input.next().is_some() {
                    Token::TextManipulationAdvanced
                } else {
                    // Incomplete find motion
                    Token::Unhandled(format!("{operator}{ch}"))
                }
            }
            // g-prefix motions (gg, gj, gk, g$, etc.)
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
            // Text objects (i/a + object)
            'i' | 'a' => {
                if let Some(&obj_ch) = self.input.peek() {
                    if Self::is_text_object_char(obj_ch) {
                        self.input.next(); // consume the object char
                        return Token::TextManipulationAdvanced;
                    }
                }
                Token::Unhandled(format!("{operator}{ch}"))
            }
            // Unrecognized motion
            _ => Token::Unhandled(format!("{operator}{ch}")),
        }
    }

    /// Handle case operator with accumulated motion count
    fn handle_case_operator_with_motion(&mut self, operator: &str, count: u32) -> Token {
        if let Some(ch) = self.input.next() {
            self.handle_case_operator_char_motion(operator, count, ch)
        } else {
            Token::Unhandled(operator.to_string())
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn next_token(&mut self) -> Option<Token> {
        // Handle accumulated state from previous calls
        // First, check if we're in command, search, or replace mode and extract content
        // 0 = CommandMode, 1 = SearchMode, 2 = ReplaceMode
        let mode_content = match &mut self.state {
            State::CommandMode { content } => Some((0, std::mem::take(content))),
            State::SearchMode { content } => Some((1, std::mem::take(content))),
            State::ReplaceMode { content } => Some((2, std::mem::take(content))),
            _ => None,
        };

        if let Some((mode_type, mut content)) = mode_content {
            // mode_type: 0 = CommandMode, 1 = SearchMode, 2 = ReplaceMode

            // ReplaceMode only terminates on |escape|
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

                        // Note: Future extension for characters that accept counts goes here
                        match ch {
                            'j' | 'k' => {
                                self.input.next(); // Consume the command
                                Some(Token::MoveVerticalBasic(i32::try_from(count).unwrap()))
                            }
                            'h' | 'l' => {
                                self.input.next();
                                Some(Token::MoveHorizontalBasic(i32::try_from(count).unwrap()))
                            }
                            'w' | 'W' | 'e' | 'E' | 'b' | 'B' => {
                                self.input.next();
                                Some(Token::MoveHorizontalChunk(i32::try_from(count).unwrap()))
                            }
                            'x' | 'J' => {
                                self.input.next();
                                Some(Token::TextManipulationBasic(i32::try_from(count).unwrap()))
                            }
                            'G' => {
                                self.input.next();
                                Some(Token::JumpToLineNumber(accumulated))
                            }
                            'g' => {
                                self.input.next(); // consume 'g'
                                                   // g-prefix commands with numeric prefix: gj, gk, gg, gJ, g~, gu, gU
                                match self.input.next() {
                                    Some('j' | 'k') => Some(Token::MoveVerticalBasic(
                                        i32::try_from(count).unwrap(),
                                    )),
                                    Some('g') => Some(Token::JumpToLineNumber(accumulated)),
                                    Some('J') => Some(Token::TextManipulationBasic(
                                        i32::try_from(count).unwrap(),
                                    )),
                                    Some('~') => {
                                        // g~ case toggle operator with count
                                        self.state = State::CaseOperatorPending {
                                            operator: "g~".to_string(),
                                            count,
                                        };
                                        self.next_token()
                                    }
                                    Some('u') => {
                                        // gu lowercase operator with count
                                        self.state = State::CaseOperatorPending {
                                            operator: "gu".to_string(),
                                            count,
                                        };
                                        self.next_token()
                                    }
                                    Some('U') => {
                                        // gU uppercase operator with count
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
                                self.input.next(); // consume 'z'
                                                   // z-prefix commands (no numeric prefix support per spec)
                                match self.input.next() {
                                    Some('z' | 't' | 'b') => Some(Token::CameraMovement),
                                    Some(ch) => Some(Token::Unhandled(format!("z{ch}"))),
                                    None => Some(Token::Unhandled("z".into())),
                                }
                            }
                            'f' | 'F' | 't' | 'T' => {
                                self.input.next(); // consume the command char
                                                   // These commands consume the next character as the target
                                if self.input.next().is_some() {
                                    Some(Token::JumpToHorizontal)
                                } else {
                                    Some(Token::Unhandled(ch.to_string()))
                                }
                            }
                            'r' => {
                                self.input.next(); // consume 'r'
                                                   // Replace command consumes the next character
                                if self.input.next().is_some() {
                                    Some(Token::TextManipulationBasic(
                                        i32::try_from(count).unwrap(),
                                    ))
                                } else {
                                    Some(Token::Unhandled("r".into()))
                                }
                            }
                            'd' => {
                                self.input.next(); // consume 'd'
                                self.state = State::OperatorPending {
                                    operator: Operator::Delete,
                                    count,
                                };
                                self.next_token()
                            }
                            'y' => {
                                self.input.next(); // consume 'y'
                                self.state = State::OperatorPending {
                                    operator: Operator::Yank,
                                    count,
                                };
                                self.next_token()
                            }
                            'c' => {
                                self.input.next(); // consume 'c'
                                self.state = State::OperatorPending {
                                    operator: Operator::Change,
                                    count,
                                };
                                self.next_token()
                            }
                            '<' => {
                                self.input.next(); // consume '<'
                                if let Some(ctrl_char) = self.try_parse_control_sequence() {
                                    Some(self.handle_control_sequence(ctrl_char, count))
                                } else {
                                    // Not a valid control sequence, return accumulated as unhandled
                                    // and let '<' be processed next time
                                    Some(Token::Unhandled(accumulated))
                                }
                            }
                            _ => {
                                // Not a command we handle with counts
                                Some(Token::Unhandled(accumulated))
                            }
                        }
                    }
                } else {
                    // End of input while accumulating
                    self.state = State::None;
                    let accumulated = self.accumulated_string.clone();
                    self.accumulated_string.clear();
                    Some(Token::Unhandled(accumulated))
                }
            }
            State::None => {
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
                    'h' | 'l' => Some(Token::MoveHorizontalBasic(1)),
                    'w' | 'W' | 'e' | 'E' | 'b' | 'B' => Some(Token::MoveHorizontalChunk(1)),
                    'u' | 'U' => Some(Token::UndoRedo),
                    '.' => Some(Token::DotRepeat),
                    'M' | 'H' | 'L' => Some(Token::JumpToVertical),
                    'p' | 'P' => Some(Token::YankPaste),
                    'x' | 'J' => Some(Token::TextManipulationBasic(1)),
                    'G' => Some(Token::JumpToLineNumber(String::new())),
                    'g' => {
                        // g-prefix commands: gj, gk, gg, gJ, g~, gu, gU
                        match self.input.next() {
                            Some('j' | 'k') => Some(Token::MoveVerticalBasic(1)),
                            Some('g') => Some(Token::JumpToLineNumber(String::new())),
                            Some('J') => Some(Token::TextManipulationBasic(1)),
                            Some('~') => {
                                // g~ case toggle operator - enter case operator pending state
                                self.state = State::CaseOperatorPending {
                                    operator: "g~".to_string(),
                                    count: 1,
                                };
                                self.next_token()
                            }
                            Some('u') => {
                                // gu lowercase operator - enter case operator pending state
                                self.state = State::CaseOperatorPending {
                                    operator: "gu".to_string(),
                                    count: 1,
                                };
                                self.next_token()
                            }
                            Some('U') => {
                                // gU uppercase operator - enter case operator pending state
                                self.state = State::CaseOperatorPending {
                                    operator: "gU".to_string(),
                                    count: 1,
                                };
                                self.next_token()
                            }
                            Some(ch) => Some(Token::Unhandled(format!("g{ch}"))),
                            None => Some(Token::Unhandled("g".into())),
                        }
                    }
                    'R' => {
                        // Enter replace mode - accumulate until <Esc>
                        self.state = State::ReplaceMode {
                            content: String::new(),
                        };
                        self.next_token()
                    }
                    'z' => {
                        // z-prefix commands: zz, zt, zb
                        match self.input.next() {
                            Some('z' | 't' | 'b') => Some(Token::CameraMovement),
                            Some(ch) => Some(Token::Unhandled(format!("z{ch}"))),
                            None => Some(Token::Unhandled("z".into())),
                        }
                    }
                    'f' | 'F' | 't' | 'T' => {
                        // These commands consume the next character as the target
                        if self.input.next().is_some() {
                            Some(Token::JumpToHorizontal)
                        } else {
                            Some(Token::Unhandled(ch.to_string()))
                        }
                    }
                    'r' => {
                        // Replace command consumes the next character
                        if self.input.next().is_some() {
                            Some(Token::TextManipulationBasic(1))
                        } else {
                            Some(Token::Unhandled("r".into()))
                        }
                    }
                    'd' => {
                        // Delete operator - enter operator pending state
                        self.state = State::OperatorPending {
                            operator: Operator::Delete,
                            count: 1,
                        };
                        self.next_token()
                    }
                    'y' => {
                        // Yank operator - enter operator pending state
                        self.state = State::OperatorPending {
                            operator: Operator::Yank,
                            count: 1,
                        };
                        self.next_token()
                    }
                    'c' => {
                        // Change operator - enter operator pending state
                        self.state = State::OperatorPending {
                            operator: Operator::Change,
                            count: 1,
                        };
                        self.next_token()
                    }
                    's' => {
                        // s is alias for cl (substitute character)
                        Some(Token::TextManipulationAdvanced)
                    }
                    'S' => {
                        // S is alias for cc (substitute line)
                        Some(Token::TextManipulationAdvanced)
                    }
                    'C' => {
                        // C is alias for c$ (change to end of line)
                        Some(Token::TextManipulationAdvanced)
                    }
                    'Y' => {
                        // Y is alias for y$ (yank to end of line)
                        Some(Token::YankPaste)
                    }
                    '%' => {
                        // Jump to matching bracket (matchit)
                        Some(Token::JumpFromContext)
                    }
                    '<' => {
                        // Try to parse control sequence
                        if let Some(ctrl_char) = self.try_parse_control_sequence() {
                            Some(self.handle_control_sequence(ctrl_char, 1))
                        } else {
                            Some(Token::Unhandled("<".into()))
                        }
                    }
                    '|' => {
                        // Try to parse pipe-delimited special key
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
                        // Enter command mode
                        self.state = State::CommandMode {
                            content: String::new(),
                        };
                        self.next_token()
                    }
                    '/' | '?' => {
                        // Enter search mode
                        self.state = State::SearchMode {
                            content: String::new(),
                        };
                        self.next_token()
                    }
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

    // Phase 1 test cases
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
        assert!(matches!(lexer.next_token(), Some(Token::JumpToLineNumber(ref s)) if s == ""));
        assert!(matches!(lexer.next_token(), Some(Token::JumpToLineNumber(ref s)) if s == "10"));
    }

    // Phase 2 test cases
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

    // Phase 3 test cases
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
        assert!(matches!(lexer.next_token(), Some(Token::JumpToLineNumber(ref s)) if s == ""));
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

    // Phase 4 test cases
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

    // Phase 5 test cases - Operator-Pending Commands (d, y, c)
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

    // Phase 6 test cases - Command Mode Sequences
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

    // Phase 7 test cases - Replace Mode and Advanced Text Manipulation
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

    // Phase 8 test cases - Special Sequences and Edge Cases
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

    //
    // #[test]
    // fn basic_vertical_movements() {
    //     const TEST_INPUT: &str = "j10jkk5kjj";
    // }
    //
    // #[test]
    // fn basic_horizontal_movements() {
    //     const TEST_INPUT: &str = "10hll5lh<Esc>h";
    // }
    //
    // #[test]
    // fn chunk_horizontal_movements() {
    //     const TEST_INPUT: &str = "10weEb5bw";
    // }
    //
    // #[test]
    // fn mixed_input_movements_hb_hc_vm() {
    //     const TEST_INPUT: &str = "jj3jwwbE3wllkk";
    // }
    //
    // #[test]
    // fn jump_horizontal_movements() {
    //     const TEST_INPUT: &str = "f3;;nFlnt3T3";
    // }
    //
    // #[test]
    // fn jump_to_line_number_gg() {
    //     const TEST_INPUT: &str = "33gg";
    // }
    //
    // #[test]
    // fn jump_to_line_number_g() {
    //     const TEST_INPUT: &str = "22Gj";
    // }
    //
    // #[test]
    // fn jump_to_line_number_command_mode() {
    //     const TEST_INPUT: &str = "j:322|enter|";
    // }
    //
    // #[test]
    // fn jump_to_line_number_command_mode_cr_issue_edition() {
    //     const TEST_INPUT: &str = "j:322|enter|j";
    // }
    //
    // #[test]
    // fn jump_to_vertical() {
    //     const TEST_INPUT: &str = "MHL<C-F><C-B>";
    // }
    //
    // #[test]
    // fn jump_from_context() {
    //     const TEST_INPUT: &str = ":<C-U>call<Space>matchit#Match_wrapper('',1,'n')|enter|m'zv";
    // }
    //
    // #[test]
    // fn camera_movement() {
    //     // 5 camera movements
    //     const TEST_INPUT: &str = "zzzzbzt<C-E><C-Y>";
    // }
    //
    // #[test]
    // fn window_management() {
    //     // 2 windows, 2 move vertical basics, 16 windows
    //     const TEST_INPUT: &str = "<C-W>s<C-W>vkk<C-W>w<C-W>q<C-W>x<C-W>=<C-W>h<C-W>j<C-W>k<C-W>l<C-W>H<C-W>L<C-W>J<C-W>K<C-H><C-J><C-K><C-L>";
    // }
    //
    // #[test]
    // fn text_manipulation_basic() {
    //     // 4 text manip basics
    //     const TEST_INPUT: &str = "12xdlJ3rp4gJ";
    // }
    //
    // #[test]
    // fn text_manipulation_advanced_1() {
    //     // 3 text manip advanced
    //     const TEST_INPUT: &str = "c$$gu3wgU44$";
    // }
    //
    // #[test]
    // fn text_manipulation_advanced_2() {
    //     // 3 text manip advanced
    //     const TEST_INPUT: &str = "Rxxx<Esc>R3<Esc>R<Esc>";
    // }
    //
    // #[test]
    // fn text_manipulation_advanced_3() {
    //     // 2 advanceds
    //     const TEST_INPUT: &str = "gu3fgguF.";
    // }
    //
    // #[test]
    // fn text_manipulation_advanced_tokens() {
    //     // 8 text manip advances
    //     const TEST_INPUT: &str = "c$$Cc$ceecwwsclSccciwwiwcawwaw";
    // }
    //
    // #[test]
    // fn text_manipulation_advanced_change_arounds() {
    //     // 6 text manip advanceds
    //     const TEST_INPUT: &str = r#"ci))<C-\><C-N>zvzvvci((<C-\><C-N>zvzvvci[[<C-\><C-N>zvzvvci]]<C-\><C-N>zvzvvci{{<C-\><C-N>zvzvvci}}<C-\><C-N>zvzvv"#;
    // }
    //
    // #[test]
    // fn yank_paste() {
    //     // 8 yank pastes
    //     const TEST_INPUT: &str = r#"3""3p""1p4""4P3y$y$yiw3yawy<Esc><C-\><C-N><Esc>"#;
    // }
    //
    // #[test]
    // fn undo_redo() {
    //     // 3 undoredo
    //     const TEST_INPUT: &str = "uU<C-R>";
    // }
    //
    // #[test]
    // fn dot_repeater() {
    //     // move chunk 3, dot repeat, move chunk 3
    //     const TEST_INPUT: &str = "3w.3w";
    // }
    //
    // #[test]
    // fn command_search() {
    //     // command search true, command search false
    //     const TEST_INPUT: &str = r#"/testsearch|enter|/testsearch2<Esc>"#;
    // }
    //
    // #[test]
    // fn delete_text() {
    //     // delete text 3, delete text 1, delete text 3, delete text 1
    //     const TEST_INPUT: &str = "d33ddddd3xx";
    // }
    //
    // #[test]
    // fn delete_text_word() {
    //     // delete text 1, delete text 3
    //     const TEST_INPUT: &str = "dwwd33ww";
    // }
    //
    // #[test]
    // fn help_page() {
    //     // help, move move, help
    //     const TEST_INPUT: &str = ":h test<Esc>jj:help test|enter|";
    // }
    //
    // #[test]
    // fn save_file() {
    //     const TEST_INPUT: &str = ":w<Esc>j:w|enter|";
    // }
    //
    // #[test]
    // fn gracefully_handles_commands() {
    //     const TEST_INPUT: &str = ":Vimscape|enter|";
    // }
    //
    // #[test]
    // fn handles_commands_gracefully_vimscape_show_data() {
    //     const TEST_INPUT: &str = ":lua<Space>require('vimscape2007').show_data()|enter|";
    // }
    //
    // #[test]
    // fn gracefully_handles_commands_with_space() {
    //     const TEST_INPUT: &str = ":Vimscape toggle<Esc>";
    // }
}

use std::{iter::Peekable, str::Chars};

use crate::token::Token;

#[derive(Debug)]
enum State {
    None,
    AccumulatingCount(u32),
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
    fn handle_control_sequence(&mut self, ctrl_char: char, count: u32) -> Option<Token> {
        match ctrl_char {
            // <C-U>, <C-D> -> MoveVerticalChunk(n)
            'U' | 'D' => Some(Token::MoveVerticalChunk(i32::try_from(count).unwrap())),

            // <C-F>, <C-B> -> JumpToVertical
            'F' | 'B' => Some(Token::JumpToVertical),

            // <C-E>, <C-Y> -> CameraMovement
            'E' | 'Y' => Some(Token::CameraMovement),

            // <C-R> -> UndoRedo
            'R' => Some(Token::UndoRedo),

            // <C-H>, <C-J>, <C-K>, <C-L> -> WindowManagement
            'H' | 'J' | 'K' | 'L' => Some(Token::WindowManagement),

            // <C-W> -> WindowManagement (consumes next character)
            'W' => {
                // Consume the next character as part of the window command
                // e.g., <C-W>s, <C-W>v, etc.
                let _ = self.input.next();
                Some(Token::WindowManagement)
            }

            // Unrecognized control sequence
            _ => Some(Token::Unhandled(format!("<C-{}>", ctrl_char))),
        }
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

    pub fn next_token(&mut self) -> Option<Token> {
        // Handle accumulated state from previous calls
        match self.state {
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
                                                   // g-prefix commands with numeric prefix: gj, gk, gg, gJ
                                match self.input.next() {
                                    Some('j') | Some('k') => Some(Token::MoveVerticalBasic(
                                        i32::try_from(count).unwrap(),
                                    )),
                                    Some('g') => Some(Token::JumpToLineNumber(accumulated)),
                                    Some('J') => Some(Token::TextManipulationBasic(
                                        i32::try_from(count).unwrap(),
                                    )),
                                    Some(ch) => Some(Token::Unhandled(format!("g{}", ch))),
                                    None => Some(Token::Unhandled("g".into())),
                                }
                            }
                            'z' => {
                                self.input.next(); // consume 'z'
                                                   // z-prefix commands (no numeric prefix support per spec)
                                match self.input.next() {
                                    Some('z') | Some('t') | Some('b') => {
                                        Some(Token::CameraMovement)
                                    }
                                    Some(ch) => Some(Token::Unhandled(format!("z{}", ch))),
                                    None => Some(Token::Unhandled("z".into())),
                                }
                            }
                            '<' => {
                                self.input.next(); // consume '<'
                                if let Some(ctrl_char) = self.try_parse_control_sequence() {
                                    self.handle_control_sequence(ctrl_char, count)
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
                    'x' => Some(Token::TextManipulationBasic(1)),
                    'J' => Some(Token::TextManipulationBasic(1)),
                    'G' => Some(Token::JumpToLineNumber(String::new())),
                    'g' => {
                        // g-prefix commands: gj, gk, gg, gJ
                        match self.input.next() {
                            Some('j') | Some('k') => Some(Token::MoveVerticalBasic(1)),
                            Some('g') => Some(Token::JumpToLineNumber(String::new())),
                            Some('J') => Some(Token::TextManipulationBasic(1)),
                            Some(ch) => Some(Token::Unhandled(format!("g{}", ch))),
                            None => Some(Token::Unhandled("g".into())),
                        }
                    }
                    'z' => {
                        // z-prefix commands: zz, zt, zb
                        match self.input.next() {
                            Some('z') | Some('t') | Some('b') => Some(Token::CameraMovement),
                            Some(ch) => Some(Token::Unhandled(format!("z{}", ch))),
                            None => Some(Token::Unhandled("z".into())),
                        }
                    }
                    '<' => {
                        // Try to parse control sequence
                        if let Some(ctrl_char) = self.try_parse_control_sequence() {
                            self.handle_control_sequence(ctrl_char, 1)
                        } else {
                            Some(Token::Unhandled("<".into()))
                        }
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

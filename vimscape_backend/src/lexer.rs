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
                            'w' | 'b' => {
                                self.input.next();
                                Some(Token::MoveHorizontalBasic(i32::try_from(count).unwrap()))
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
                    'w' | 'b' => Some(Token::MoveHorizontalBasic(1)),
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
        let mut lexer = Lexer::new("j5kx");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(1))
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(5))
        ));
        assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "x"));
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
        assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "12"));
        assert!(matches!(lexer.next_token(), Some(Token::Unhandled(ref s)) if s == "x"));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(34))
        ));
        assert!(lexer.next_token().is_none());
    }

    #[test]
    fn test_move_horizontal_and_move_vertical() {
        let mut lexer = Lexer::new("2w3kbj");
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveHorizontalBasic(2))
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveVerticalBasic(3))
        ));
        assert!(matches!(
            lexer.next_token(),
            Some(Token::MoveHorizontalBasic(1))
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

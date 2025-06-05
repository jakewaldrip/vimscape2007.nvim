use std::{iter::Peekable, str::Chars};

use crate::token::Token;

enum State {
    None,
}

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    state: State,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let chars = input.chars().peekable();
        Self {
            input: chars,
            state: State::None,
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        let ch = self.input.next()?;

        // Check for ascii digits
        if ch.is_ascii_digit() {
            // peek until theres no more digits
            // consume this string
            // convert to number
            // switch state to Modifying
            // allow match to handle next character as normal, with state adjusted
        }

        let token = match ch {
            'j' => Token::MoveVerticalBasic(1),
            'k' => Token::MoveVerticalBasic(1),
            _ => Token::Unhandled(ch.into()),
        };
        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wip_test() {
        let src = "10jkj";
        let mut lexer = Lexer::new(src);

        println!("Source: {src}");
        while let Some(token) = lexer.next_token() {
            println!("Output: {token:?}");
        }
    }

    // #[test]
    // fn no_input_as_none() {
    //     const TEST_INPUT: &str = "";
    // }
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

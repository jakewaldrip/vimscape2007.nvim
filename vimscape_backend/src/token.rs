// TODO: Check what a <leader>X renders as
#[derive(Debug, PartialEq)]
pub enum Token {
    // gj, gk, j, k, 10j,
    MoveVerticalBasic(i32),

    // 10h, h, l
    MoveHorizontalBasic(i32),

    // 10<C-U>, <C-U>, <C-D>
    MoveVerticalChunk,

    // 10w, w, W, e, E, b, B
    MoveHorizontalChunk(i32),

    // _g_, F, f, T, t, + any 1 char
    JumpToHorizontal,

    // :10|enter|, 10gg, gg, G
    JumpToLineNumber(i32),

    // M, H, L, <C-F>, <C-B>
    JumpToVertical,

    // % renders as the token below
    // :<C-U>call<Space>matchit#Match_wrapper('',1,'n')|enter|m'zv
    JumpFromContext,

    // zz renders as zzz
    // zb, zt, zzz, <C-E>, <C-Y>
    CameraMovement,

    // <C-W>(svwqx=hljkHLJK), <C-H>, <C-J>, <C-K>, <C-L>
    WindowManagement,

    // x renders as xdl
    // [num](xdl, gJ, J, r[character])
    TextManipulationBasic(i32),

    // R[character]<Esc>, g(~uU)[num](wWeEbB$^0fFtT),
    // ci))<C-\><C-N>zvzvv
    // ci((<C-\><C-N>zvzvv
    // ci[[<C-\><C-N>zvzvv
    // ci]]<C-\><C-N>zvzvv
    // ci{{<C-\><C-N>zvzvv
    // ci}}<C-\><C-N>zvzvv
    // c$$, cC$, cee, cww, scl, Scc, ciwwiw, cawwaw
    TextManipulationAdvanced,

    // p renders as ""1p
    // yy renders as yyy
    // Y renders as y$
    // y<Esc> renders as y<Esc><C-\><C-N><Esc>, i can't explain that one
    // [num]""(p, P), [num]y($, w, iw, aw, yy), [num]y<Esc><C-\><C-N><Esc>
    YankPaste,

    // u, U, <C-R>
    UndoRedo,

    // This literally just repeats the keys
    // so i will just let it grant xp for both categories
    // .
    DotRepeat,

    // /[any characters]
    CommandSearch(bool),

    // d$, dw, etc, doubles up on every key, ie dww, d$$, etc
    // If a number is included, it doubles the digits of the number
    // [num doubled last digit](dww, d$$, dWW, dee, dEE, dbb, dBB, d$$, d^^, d00)
    DeleteText(i32),

    // :[any characters]
    Command(bool),

    // :(h, help)[any characters](|enter|, <Esc>)
    HelpPage(bool),

    // :w(|enter|, <Esc>)
    SaveFile(bool),

    // Anything else can go here?
    Unhandled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_input_as_none() {
        const TEST_INPUT: &str = "";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn basic_vertical_movements() {
        const TEST_INPUT: &str = "j10jkk5kjj";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(1))));
        assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(10))));
        assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(1))));
        assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(1))));
        assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(5))));
        assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(1))));
        assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(1))));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn basic_horizontal_movements() {
        const TEST_INPUT: &str = "10hll5lh<Esc>h";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalBasic(10))));
        assert_eq!(lexer.slice(), "10h");
        assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalBasic(1))));
        assert_eq!(lexer.slice(), "l");
        assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalBasic(1))));
        assert_eq!(lexer.slice(), "l");
        assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalBasic(5))));
        assert_eq!(lexer.slice(), "5l");
        assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalBasic(1))));
        assert_eq!(lexer.slice(), "h");
        assert_eq!(lexer.next(), Some(Ok(Token::Unhandled)));
        assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalBasic(1))));
        assert_eq!(lexer.slice(), "h");
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn chunk_horizontal_movements() {
        const TEST_INPUT: &str = "10weEb5bw";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalChunk(10))));
        assert_eq!(lexer.slice(), "10w");
        assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalChunk(1))));
        assert_eq!(lexer.slice(), "e");
        assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalChunk(1))));
        assert_eq!(lexer.slice(), "E");
        assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalChunk(1))));
        assert_eq!(lexer.slice(), "b");
        assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalChunk(5))));
        assert_eq!(lexer.slice(), "5b");
        assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalChunk(1))));
        assert_eq!(lexer.slice(), "w");
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn mixed_input_movements_hb_hc_vm() {
        const TEST_INPUT: &str = "jj3jwwbE3wllkk";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(1))));
        assert_eq!(lexer.slice(), "j");
        assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(1))));
        assert_eq!(lexer.slice(), "j");
        assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(3))));
        assert_eq!(lexer.slice(), "3j");
        assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalChunk(1))));
        assert_eq!(lexer.slice(), "w");
        assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalChunk(1))));
        assert_eq!(lexer.slice(), "w");
        assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalChunk(1))));
        assert_eq!(lexer.slice(), "b");
        assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalChunk(1))));
        assert_eq!(lexer.slice(), "E");
        assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalChunk(3))));
        assert_eq!(lexer.slice(), "3w");
        assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalBasic(1))));
        assert_eq!(lexer.slice(), "l");
        assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalBasic(1))));
        assert_eq!(lexer.slice(), "l");
        assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(1))));
        assert_eq!(lexer.slice(), "k");
        assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(1))));
        assert_eq!(lexer.slice(), "k");
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn jump_horizontal_movements() {
        const TEST_INPUT: &str = "f3;;nFlnt3T3";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::JumpToHorizontal)));
        assert_eq!(lexer.slice(), "f3;;n");
        assert_eq!(lexer.next(), Some(Ok(Token::JumpToHorizontal)));
        assert_eq!(lexer.slice(), "Fln");
        assert_eq!(lexer.next(), Some(Ok(Token::JumpToHorizontal)));
        assert_eq!(lexer.slice(), "t3");
        assert_eq!(lexer.next(), Some(Ok(Token::JumpToHorizontal)));
        assert_eq!(lexer.slice(), "T3");
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn jump_to_line_number_gg() {
        const TEST_INPUT: &str = "33gg";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::JumpToLineNumber(33))));
        assert_eq!(lexer.slice(), TEST_INPUT);
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn jump_to_line_number_g() {
        const TEST_INPUT: &str = "22Gj";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::JumpToLineNumber(22))));
        assert_eq!(lexer.slice(), "22G");
        assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(1))));
        assert_eq!(lexer.slice(), "j");
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn jump_to_line_number_command_mode() {
        const TEST_INPUT: &str = "j:322|enter|";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(1))));
        assert_eq!(lexer.next(), Some(Ok(Token::JumpToLineNumber(322))));
        assert_eq!(lexer.slice(), ":322|enter|");
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn jump_to_line_number_command_mode_cr_issue_edition() {
        const TEST_INPUT: &str = "j:322|enter|j";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(1))));
        assert_eq!(lexer.next(), Some(Ok(Token::JumpToLineNumber(322))));
        assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(1))));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn jump_to_vertical() {
        const TEST_INPUT: &str = "MHL<C-F><C-B>";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::JumpToVertical)));
        assert_eq!(lexer.next(), Some(Ok(Token::JumpToVertical)));
        assert_eq!(lexer.next(), Some(Ok(Token::JumpToVertical)));
        assert_eq!(lexer.next(), Some(Ok(Token::JumpToVertical)));
        assert_eq!(lexer.next(), Some(Ok(Token::JumpToVertical)));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn jump_from_context() {
        const TEST_INPUT: &str = ":<C-U>call<Space>matchit#Match_wrapper('',1,'n')|enter|m'zv";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::JumpFromContext)));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn camera_movement() {
        const TEST_INPUT: &str = "zzzzbzt<C-E><C-Y>";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::CameraMovement)));
        assert_eq!(lexer.next(), Some(Ok(Token::CameraMovement)));
        assert_eq!(lexer.next(), Some(Ok(Token::CameraMovement)));
        assert_eq!(lexer.next(), Some(Ok(Token::CameraMovement)));
        assert_eq!(lexer.next(), Some(Ok(Token::CameraMovement)));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn window_management() {
        const TEST_INPUT: &str = "<C-W>s<C-W>vkk<C-W>w<C-W>q<C-W>x<C-W>=<C-W>h<C-W>j<C-W>k<C-W>l<C-W>H<C-W>L<C-W>J<C-W>K<C-H><C-J><C-K><C-L>";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::WindowManagement)));
        assert_eq!(lexer.next(), Some(Ok(Token::WindowManagement)));
        assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(1))));
        assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(1))));
        assert_eq!(lexer.next(), Some(Ok(Token::WindowManagement)));
        assert_eq!(lexer.next(), Some(Ok(Token::WindowManagement)));
        assert_eq!(lexer.next(), Some(Ok(Token::WindowManagement)));
        assert_eq!(lexer.next(), Some(Ok(Token::WindowManagement)));
        assert_eq!(lexer.next(), Some(Ok(Token::WindowManagement)));
        assert_eq!(lexer.next(), Some(Ok(Token::WindowManagement)));
        assert_eq!(lexer.next(), Some(Ok(Token::WindowManagement)));
        assert_eq!(lexer.next(), Some(Ok(Token::WindowManagement)));
        assert_eq!(lexer.next(), Some(Ok(Token::WindowManagement)));
        assert_eq!(lexer.next(), Some(Ok(Token::WindowManagement)));
        assert_eq!(lexer.next(), Some(Ok(Token::WindowManagement)));
        assert_eq!(lexer.next(), Some(Ok(Token::WindowManagement)));
        assert_eq!(lexer.next(), Some(Ok(Token::WindowManagement)));
        assert_eq!(lexer.next(), Some(Ok(Token::WindowManagement)));
        assert_eq!(lexer.next(), Some(Ok(Token::WindowManagement)));
        assert_eq!(lexer.next(), Some(Ok(Token::WindowManagement)));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn text_manipulation_basic() {
        const TEST_INPUT: &str = "12xdlJ3rp4gJ";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationBasic(12))));
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationBasic(1))));
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationBasic(3))));
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationBasic(4))));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn text_manipulation_advanced_1() {
        const TEST_INPUT: &str = "c$$gu3wgU44$";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn text_manipulation_advanced_2() {
        const TEST_INPUT: &str = "Rxxx<Esc>R3<Esc>R<Esc>";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.slice(), "Rxxx<Esc>");
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn text_manipulation_advanced_3() {
        const TEST_INPUT: &str = "gu3fgguF.";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.slice(), "gu3fg");
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.slice(), "guF.");
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn text_manipulation_advanced_tokens() {
        const TEST_INPUT: &str = "c$$Cc$ceecwwsclSccciwwiwcawwaw";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.slice(), "c$$");
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.slice(), "Cc$");
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.slice(), "cee");
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.slice(), "cww");
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.slice(), "scl");
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.slice(), "Scc");
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.slice(), "ciwwiw");
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.slice(), "cawwaw");
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn text_manipulation_advanced_change_arounds() {
        const TEST_INPUT: &str = r#"ci))<C-\><C-N>zvzvvci((<C-\><C-N>zvzvvci[[<C-\><C-N>zvzvvci]]<C-\><C-N>zvzvvci{{<C-\><C-N>zvzvvci}}<C-\><C-N>zvzvv"#;
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.next(), Some(Ok(Token::TextManipulationAdvanced)));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn yank_paste() {
        const TEST_INPUT: &str = r#"3""3p""1p4""4P3y$y$yiw3yawy<Esc><C-\><C-N><Esc>"#;
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::YankPaste)));
        assert_eq!(lexer.slice(), r#"3""3p"#);
        assert_eq!(lexer.next(), Some(Ok(Token::YankPaste)));
        assert_eq!(lexer.slice(), r#"""1p"#);
        assert_eq!(lexer.next(), Some(Ok(Token::YankPaste)));
        assert_eq!(lexer.slice(), r#"4""4P"#);
        assert_eq!(lexer.next(), Some(Ok(Token::YankPaste)));
        assert_eq!(lexer.slice(), "3y$");
        assert_eq!(lexer.next(), Some(Ok(Token::YankPaste)));
        assert_eq!(lexer.slice(), "y$");
        assert_eq!(lexer.next(), Some(Ok(Token::YankPaste)));
        assert_eq!(lexer.slice(), "yiw");
        assert_eq!(lexer.next(), Some(Ok(Token::YankPaste)));
        assert_eq!(lexer.slice(), "3yaw");
        assert_eq!(lexer.next(), Some(Ok(Token::YankPaste)));
        assert_eq!(lexer.slice(), r#"y<Esc><C-\><C-N><Esc>"#);
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn undo_redo() {
        const TEST_INPUT: &str = "uU<C-R>";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::UndoRedo)));
        assert_eq!(lexer.slice(), "u");
        assert_eq!(lexer.next(), Some(Ok(Token::UndoRedo)));
        assert_eq!(lexer.slice(), "U");
        assert_eq!(lexer.next(), Some(Ok(Token::UndoRedo)));
        assert_eq!(lexer.slice(), "<C-R>");
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn dot_repeater() {
        const TEST_INPUT: &str = "3w.3w";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalChunk(3))));
        assert_eq!(lexer.next(), Some(Ok(Token::DotRepeat)));
        assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalChunk(3))));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn command_search() {
        const TEST_INPUT: &str = r#"/testsearch|enter|/testsearch2<Esc>"#;
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::CommandSearch(false))));
        assert_eq!(lexer.next(), Some(Ok(Token::CommandSearch(true))));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn delete_text() {
        const TEST_INPUT: &str = "d33ddddd3xx";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::DeleteText(3))));
        assert_eq!(lexer.slice(), "d33dd");
        assert_eq!(lexer.next(), Some(Ok(Token::DeleteText(1))));
        assert_eq!(lexer.slice(), "ddd");
        assert_eq!(lexer.next(), Some(Ok(Token::DeleteText(3))));
        assert_eq!(lexer.slice(), "3x");
        assert_eq!(lexer.next(), Some(Ok(Token::DeleteText(1))));
        assert_eq!(lexer.slice(), "x");
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn delete_text_word() {
        const TEST_INPUT: &str = "dwwd33ww";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::DeleteText(1))));
        assert_eq!(lexer.slice(), "dww");
        assert_eq!(lexer.next(), Some(Ok(Token::DeleteText(3))));
        assert_eq!(lexer.slice(), "d33ww");
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn unhandled_tokens() {
        const TEST_INPUT: &str = ">|enter|<Esc>";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::Unhandled)));
        assert_eq!(lexer.slice(), ">");
        assert_eq!(lexer.next(), Some(Ok(Token::Unhandled)));
        assert_eq!(lexer.slice(), "|enter|");
        assert_eq!(lexer.next(), Some(Ok(Token::Unhandled)));
        assert_eq!(lexer.slice(), "<Esc>");
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn help_page() {
        const TEST_INPUT: &str = ":h test<Esc>jj:help test|enter|";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::HelpPage(true))));
        assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(1))));
        assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(1))));
        assert_eq!(lexer.next(), Some(Ok(Token::HelpPage(false))));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn save_file() {
        const TEST_INPUT: &str = ":w<Esc>j:w|enter|";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::SaveFile(true))));
        assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(1))));
        assert_eq!(lexer.next(), Some(Ok(Token::SaveFile(false))));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn gracefully_handles_commands() {
        const TEST_INPUT: &str = ":Vimscape|enter|";
        let mut lexer = Token::lexer(TEST_INPUT);

        assert_eq!(lexer.next(), Some(Ok(Token::Command(false))));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn handles_commands_gracefully_vimscape_show_data() {
        const TEST_INPUT: &str = ":lua<Space>require('vimscape2007').show_data()|enter|";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::Command(false))));
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn gracefully_handles_commands_with_space() {
        const TEST_INPUT: &str = ":Vimscape toggle<Esc>";
        let mut lexer = Token::lexer(TEST_INPUT);
        assert_eq!(lexer.next(), Some(Ok(Token::Command(true))));
        assert_eq!(lexer.next(), None);
    }
}

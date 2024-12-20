use logos::{Lexer, Logos};

// TODO figure out macros
// TODO Consider marks?
// TODO consider buffers?
// TODO figure out command mode magic
// TODO figure out save file
// Potentially this could have something to do with the colon?
// TODO figure out visual mode
// TODO for command search, consider reading in n/N keys until esc is pressed
#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    #[regex(
        r"(?:[1-9]{1}\d{0,})?(?:[jk]|gj|gk)",
        pull_modifier_from_single_movement
    )]
    MoveVerticalBasic(i32),

    #[regex(r"(?:[1-9]{1}\d{0,})?[hl]", pull_modifier_from_single_movement)]
    MoveHorizontalBasic(i32),

    #[regex(r"(?:[1-9]{1}\d{0,})?(?:<C-U>|<C-D>)")]
    MoveVerticalChunk,

    #[regex(r"(?:[1-9]{1}\d{0,})?[wWeEbB]", pull_modifier_from_single_movement)]
    MoveHorizontalChunk(i32),

    #[regex(r"[FfTt].(?:[;n]{1,})?")]
    #[token("_g_")]
    JumpToHorizontal,

    #[regex(r"(?:[1-9]{1}\d{0,})?(?:gg|G)", pull_modifier_from_arbitrary_location)]
    #[regex(
        r":[1-9]{1}\d{0,}\|enter\|",
        pull_modifier_from_arbitrary_location,
        priority = 20
    )]
    JumpToLineNumber(i32),

    #[regex(r"[MHL]")]
    #[regex(r"(?:<C-F>|<C-B>)")]
    JumpToVertical,

    // % renders as the token below
    #[token(":<C-U>call<Space>matchit#Match_wrapper('',1,'n')|enter|m'zv")]
    JumpFromContext,

    // zz renders as zzz
    #[regex(r"z(?:zz|[bt])")]
    #[regex(r"(?:<C-E>|<C-Y>)")]
    CameraMovement,

    #[regex(r"(?:<C-W>[svwqx=hljkHLJK])|<C-H>|<C-J>|<C-K>|<C-L>")]
    WindowManagement,

    // x renders as xdl
    #[regex(r"(?:[1-9]{1}\d{0,})?xdl", pull_modifier_from_arbitrary_location)]
    #[regex(r"(?:[1-9]{1}\d{0,})?J", pull_modifier_from_arbitrary_location)]
    #[regex(r"(?:[1-9]{1}\d{0,})?gJ", pull_modifier_from_arbitrary_location)]
    #[regex(r"(?:[1-9]{1}\d{0,})?r.", pull_modifier_from_arbitrary_location)]
    TextManipulationBasic(i32),

    #[regex(r"R[A-Za-z0-9]{0,}<Esc>")]
    #[regex(r"g[~uU](?:[1-9]{1}\d{0,})?[wWeEbB$^0]", priority = 20)]
    #[regex(r"g[~uU](?:[1-9]{1}\d{0,})?[fFtT].", priority = 20)]
    #[regex(r"ci\)\)<C-\\><C-N>zvzvv")]
    #[regex(r"ci\(\(<C-\\><C-N>zvzvv")]
    #[regex(r"ci\[\[<C-\\><C-N>zvzvv")]
    #[regex(r"ci\]\]<C-\\><C-N>zvzvv")]
    #[regex(r"ci\{\{<C-\\><C-N>zvzvv")]
    #[regex(r"ci\}\}<C-\\><C-N>zvzvv")]
    #[token("c$$")]
    #[token("Cc$")]
    #[regex("c(?:ee|ww)")]
    #[token("scl")]
    #[token("Scc")]
    #[token("ciwwiw")]
    #[token("cawwaw")]
    TextManipulationAdvanced,

    // p renders as ""1p
    // yy renders as yyy
    // Y renders as y$
    // y<Esc> renders as y<Esc><C-\><C-N><Esc>, i can't explain that one
    #[regex(r#"(?:[1-9]{1}\d{0,})?(?:""(?:[1-9]{1}\d{0,})?p|""(?:[1-9]{1}\d{0,})?P)"#)]
    #[regex(r"(?:[1-9]{1}\d{0,})?yyy")]
    #[regex(r"(?:[1-9]{1}\d{0,})?y(?:[$w]|iw|aw|<Esc><C-\\><C-N><Esc>)")]
    YankPaste,

    #[regex(r"(?:[uU]|<C-R>)")]
    UndoRedo,

    // This literally just repeats the keys
    // so i will just let it grant xp for both categories
    #[token(".")]
    DotRepeat,

    // Needs tests
    #[regex(r"/[A-Za-z0-9]{1,}(?:\|enter\||<Esc>)", was_command_escaped)]
    CommandSearch(bool),

    // d$, dw, etc, doubles up on every key, ie dww, d$$, etc
    // If a number is included, it doubles the digits of the number
    #[regex(
        r"d(?:[1-9]{1}\d{0,})?[dwWeEbB$^0][dwWeEbB$^0]",
        pull_modifier_from_arbitrary_location_hacky_version
    )]
    #[regex(r"(?:[1-9]{1}\d{0,})?x", pull_modifier_from_single_movement)]
    DeleteText(i32),

    #[regex(r":(?:h|help) [a-zA-z0-9]{0,}(?:\|enter\||<Esc>)", was_command_escaped)]
    HelpPage(bool),

    #[regex(r#":w(?:\|enter\||<Esc>)"#, was_command_escaped)]
    SaveFile(bool),

    #[regex(r"[a-zA-Z0-9_:<>]", priority = 1)]
    #[token("|enter|", priority = 1)]
    #[token("<Esc>", priority = 1)]
    UnhandledToken,
}

// This is used if we have a match fail, but can match a more specific regex and find it
// https://github.com/maciejhirsz/logos/issues/315
#[derive(Logos, Debug, PartialEq)]
enum RecoveryToken {}

fn pull_modifier_from_single_movement(lex: &mut Lexer<Token>) -> Option<i32> {
    let slice = lex.slice();
    let modifier = slice[..slice.len() - 1].parse().ok();
    match modifier {
        Some(num) => Some(num),
        None => Some(1),
    }
}

fn pull_modifier_from_arbitrary_location(lex: &mut Lexer<Token>) -> Option<i32> {
    let slice = lex.slice();
    let digits: String = slice.chars().filter(|char| char.is_digit(10)).collect();
    let value: Option<i32> = digits.parse().ok();
    match value {
        Some(num) => Some(num),
        None => Some(1),
    }
}

// d2d and d12d render as d22d and d112d respectively
// cut off first char if more than 1 exists
fn pull_modifier_from_arbitrary_location_hacky_version(lex: &mut Lexer<Token>) -> Option<i32> {
    let slice = lex.slice();
    let mut digit_vec: Vec<char> = slice.chars().filter(|char| char.is_digit(10)).collect();
    if digit_vec.len() == 0 {
        return Some(1);
    } else if digit_vec.len() > 1 {
        digit_vec.remove(0);
    }

    let digits: String = digit_vec.into_iter().collect();
    let value: Option<i32> = digits.parse().ok();
    match value {
        Some(num) => Some(num),
        None => Some(1),
    }
}

fn was_command_escaped(lex: &mut Lexer<Token>) -> Option<bool> {
    let slice = lex.slice();
    let was_escaped = slice.contains("<Esc>");
    Some(was_escaped)
}

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
    assert_eq!(lexer.next(), Some(Ok(Token::UnhandledToken)));
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
    assert_eq!(lexer.next(), Some(Ok(Token::UnhandledToken)));
    assert_eq!(lexer.slice(), ">");
    assert_eq!(lexer.next(), Some(Ok(Token::UnhandledToken)));
    assert_eq!(lexer.slice(), "|enter|");
    assert_eq!(lexer.next(), Some(Ok(Token::UnhandledToken)));
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

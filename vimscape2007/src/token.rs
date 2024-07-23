use logos::{Lexer, Logos};
use nvim_oxi::{self as oxi};

// TODO Figure out how it renders
// TODO Test callback methods individually? Woudn't hurt
// TODO figure out what to do with gd, gD, etc. Maybe capture them specifically?
// TODO figure out macros
#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    // tested locally, keys are correct
    #[regex(
        r"(?:[1-9]{1}\d{0,})?(?:[jk]|gj|gk)",
        pull_modifier_from_single_movement
    )]
    MoveVerticalBasic(i32),

    // Tested locally, keys are correct
    #[regex(r"(?:[1-9]{1}\d{0,})?[hl]", pull_modifier_from_single_movement)]
    MoveHorizontalBasic(i32),

    // Tested locally, keys are correct
    #[regex(r"(?:[1-9]{1}\d{0,})?(?:<C-U>|<C-D>)")]
    MoveVerticalChunk,

    // handle gE and ge, as well as g_ (end of line non blank)
    #[regex(r"(?:[1-9]{1}\d{0,})?[wWeEbB]", pull_modifier_from_single_movement)]
    MoveHorizontalChunk(i32),

    #[regex(r"[FfTt].(?:[;n]{1,})?")]
    JumpToHorizontal,

    // Needs Tests
    #[regex(r"(?:[1-9]{1}\d{0,})?(?:gg|G)", pull_modifier_from_single_movement)]
    #[regex(r":(?:[1-9]{1}\d{0,})", pull_modifier_from_arbitrary_location)]
    JumpToLineNumber(i32),

    // Needs tests
    // Tested locally, keys are correct
    #[regex(r"[MHL]")]
    #[regex(r"(?:<C-F>|<C-B>)")]
    JumpToVertical,

    // Needs Tests
    #[token("%")]
    JumpFromContext,

    // Needs tests
    // zz renders as zzz
    // Tested locally, keys are correct
    #[regex(r"z(?:zz|[bt])")]
    #[regex(r"(?:<C-E>|<C-Y>)")]
    CameraMovement,

    // Needs tests
    // Stuff like splitting windows, closing them, jumping between windows
    #[regex(r"TODO1")]
    WindowManagement,

    // Needs tests
    // Anything in visual mode will go here, maybe split this out more into basic + advanced?
    #[regex(r"TODO2")]
    VisualModeMagic,

    // Needs tests
    #[regex(r":.{1,}<CR>", was_command_completed)]
    CommandModeMagic(bool),

    // Needs tests
    // x renders as xdl
    // Tested locally, keys are correct
    #[regex(r"(?:[1-9]{1}\d{0,})?xdl")]
    #[regex(r"(?:[1-9]{1}\d{0,})?J")]
    #[regex(r"(?:[1-9]{1}\d{0,})?r.")]
    TextManipulationBasic,

    // Needs tests
    // Stuff like toggling case, replacing words, etc
    // g{} and c{} related stuff
    #[regex(r"TODO4")]
    TextManipulationAdvanced,

    // Needs tests
    // p renders as ""1p
    // yy renders as yyy
    // Y renders as y$
    // y<Esc> renders as y<Esc><C-\><C-N><Esc>, i can't explain that one
    // Tested locally, keys are correct
    #[regex(r#"(?:[1-9]{1}\d{0,})?(?:""(?:[1-9]{1}\d{0,})?p|""(?:[1-9]{1}\d{0,})?P)"#)]
    #[regex(r"(?:[1-9]{1}\d{0,})?yyy")]
    #[regex(r"(?:[1-9]{1}\d{0,})?y(?:[$w]|iw|aw|<Esc><C-\\><C-N><Esc>)")]
    YankPaste,

    // Needs tests
    // Tested locally, keys are correct
    #[regex(r"(?:[uU]|<C-R>)")]
    UndoRedo,

    // Needs tests
    // This literally just repeats the keys
    // so i will just let it grant xp for both categories
    // Tested locally, keys are correct
    #[token(".")]
    DotRepeat,

    // Needs tests
    // Make sure enter is actually <CR>
    // Tested locally, keys are correct
    #[regex(r"/.{1,}(?:<CR>|<Esc>)", was_command_completed)]
    CommandSearch(bool),

    // Needs tests
    // Tested Locally, keys correct
    // d$, dw, etc, doubles up on every key, ie dww, d$$, etc
    // If a number is included, it doubles the number instead
    #[regex(r"(?:[1-9]{1}\d{0,})?d", pull_modifier_from_single_movement)]
    #[regex(
        r"d(?:[1-9]{1}\d{0,})?[dwWeEbB$^0][dwWeEbB$^0]",
        pull_modifier_from_arbitrary_location_hacky_version
    )]
    #[regex(r"(?:[1-9]{1}\d{0,})?x", pull_modifier_from_single_movement)]
    DeleteText(i32),

    // Tested locally, keys are correct
    #[token(":w<CR>")]
    SaveFile,

    // Needs tests
    // Tested Locally, keys correct
    #[regex(
        ":(?:help|h) .{1,}(?:<CR>|<Esc>)(?:<C-W>(?:c)?)?",
        was_command_completed
    )]
    HelpPage(bool),
}

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

fn was_command_completed(lex: &mut Lexer<Token>) -> Option<bool> {
    let slice = lex.slice();
    let was_escaped = slice.contains("<Esc>");
    Some(!was_escaped)
}

#[oxi::test]
fn basic_vertical_movements() {
    const TEST_INPUT: &str = "10jkk5kj";
    let mut lexer = Token::lexer(TEST_INPUT);
    assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(10))));
    assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(1))));
    assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(1))));
    assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(5))));
    assert_eq!(lexer.next(), Some(Ok(Token::MoveVerticalBasic(1))));
    assert_eq!(lexer.next(), None);
}

#[oxi::test]
fn basic_horizontal_movements() {
    const TEST_INPUT: &str = "10hll5lh";
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
    assert_eq!(lexer.next(), None);
}

#[oxi::test]
fn chunk_horizontal_movements() {
    const TEST_INPUT: &str = "10weEb5b";
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
    assert_eq!(lexer.next(), None);
}

#[oxi::test]
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

#[oxi::test]
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

#[oxi::test]
fn save_file_token() {
    const TEST_INPUT: &str = ":w<CR>";
    let mut lexer = Token::lexer(TEST_INPUT);
    assert_eq!(lexer.next(), Some(Ok(Token::SaveFile)));
    assert_eq!(lexer.slice(), TEST_INPUT);
    assert_eq!(lexer.next(), None);
}

#[oxi::test]
fn help_page_token_command_completed() {
    const TEST_INPUT: &str = ":help vimscape2007<CR>";
    let mut lexer = Token::lexer(TEST_INPUT);
    assert_eq!(lexer.next(), Some(Ok(Token::HelpPage(true))));
    assert_eq!(lexer.slice(), TEST_INPUT);
    assert_eq!(lexer.next(), None);
}

#[oxi::test]
fn help_page_token_command_completed_false() {
    const TEST_INPUT: &str = ":help vimscape2007<Esc>";
    let mut lexer = Token::lexer(TEST_INPUT);
    assert_eq!(lexer.next(), Some(Ok(Token::HelpPage(false))));
    assert_eq!(lexer.slice(), TEST_INPUT);
    assert_eq!(lexer.next(), None);
}

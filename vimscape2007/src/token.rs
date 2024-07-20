use logos::{Lexer, Logos};
use nvim_oxi::{self as oxi};

#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    #[regex(r"(?:[1-9]{1}\d{0,})?[jk]", pull_modifier_from_single_movement)]
    MoveVerticalBasic(i32),

    #[regex(r"(?:[1-9]{1}\d{0,})?[hl]", pull_modifier_from_single_movement)]
    MoveHorizontalBasic(i32),

    #[regex(r"TODO1")]
    MoveVerticalChunk,

    #[regex(r"(?:[1-9]{1}\d{0,})?[wWeEbB]", pull_modifier_from_single_movement)]
    MoveHorizontalChunk(i32),

    #[regex(r"[Ff].(?:[;n]{1,})?")]
    JumpToHorizontal,

    // Needs Tests
    #[regex(r"TODO2")]
    JumpToVertical,

    // Needs Tests
    #[token("%")]
    JumpFromContext,

    // Needs tests
    #[regex(r"TODO3")]
    CameraMovement,

    // Needs tests
    #[regex(r"TODO4")]
    WindowManagement,

    // Needs tests
    #[regex(r"TODO5")]
    VisualModeMagic,

    // Needs tests
    #[regex(r"TODO6")]
    CommandModeMagic,

    // Needs tests
    #[regex(r"TODO7")]
    TextManipulationBasic,

    // Needs tests
    #[regex(r"TODO8")]
    TextManipulationAdvanced,

    // Needs tests
    #[regex(r"(?:[1-9]{1}\d{0,})?d", pull_modifier_from_single_movement)]
    #[regex(
        r"d(?:[1-9]{1}\d{0,})?[dwWeEbB$^0]",
        pull_modifier_from_arbitrary_location
    )]
    #[regex(r"(?:[1-9]{1}\d{0,})?x", pull_modifier_from_single_movement)]
    DeleteText(i32),

    #[token(":w<CR>")]
    SaveFile,
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
    const TEST_INPUT: &str = "f3;;nfln";
    let mut lexer = Token::lexer(TEST_INPUT);
    assert_eq!(lexer.next(), Some(Ok(Token::JumpToHorizontal)));
    assert_eq!(lexer.slice(), "f3;;n");
    assert_eq!(lexer.next(), Some(Ok(Token::JumpToHorizontal)));
    assert_eq!(lexer.slice(), "fln");
    assert_eq!(lexer.next(), None);
}

#[oxi::test]
fn save_file_token() {
    const TEST_INPUT: &str = ":w<CR>";
    let mut lexer = Token::lexer(TEST_INPUT);
    assert_eq!(lexer.next(), Some(Ok(Token::SaveFile)));
    assert_eq!(lexer.slice(), TEST_INPUT);
}

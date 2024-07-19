use logos::{Lexer, Logos};
use nvim_oxi::{self as oxi};

#[derive(Logos, Debug, PartialEq)]
pub enum Token {
    #[regex(r"(?:\d{1,})?[jk]", pull_modifier_from_single_movement)]
    MoveVerticalBasic(i32),

    #[regex(r"(?:\d{1,})?[hl]", pull_modifier_from_single_movement)]
    MoveHorizontalBasic(i32),

    #[regex(r"(?:\d{1,})?[wWeEbB]", pull_modifier_from_single_movement)]
    MoveHorizontalChunk(i32),
    // #[regex(r"<t_..>")]
    // UserCommand,
}

fn pull_modifier_from_single_movement(lex: &mut Lexer<Token>) -> Option<i32> {
    let slice = lex.slice();
    let modifier = slice[..slice.len() - 1].parse().ok();
    match modifier {
        Some(num) => Some(num),
        None => Some(1),
    }
}

#[oxi::test]
fn lexer_succeeds_basic_vertical_movements() {
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
fn lexer_succeeds_basic_horizontal_movements() {
    const TEST_INPUT: &str = "10hll5lh";
    let mut lexer = Token::lexer(TEST_INPUT);
    assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalBasic(10))));
    assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalBasic(1))));
    assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalBasic(1))));
    assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalBasic(5))));
    assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalBasic(1))));
    assert_eq!(lexer.next(), None);
}

#[oxi::test]
fn lexer_succeeds_chunk_horizontal_movements() {
    const TEST_INPUT: &str = "10weEb5b";
    let mut lexer = Token::lexer(TEST_INPUT);
    assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalChunk(10))));
    assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalChunk(1))));
    assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalChunk(1))));
    assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalChunk(1))));
    assert_eq!(lexer.next(), Some(Ok(Token::MoveHorizontalChunk(5))));
    assert_eq!(lexer.next(), None);
}

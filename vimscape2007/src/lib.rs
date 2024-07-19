use crate::token::Token;
use logos::Logos;
use nvim_oxi::{self as oxi, print, Dictionary, Function};

mod token;

#[nvim_oxi::plugin]
fn vimscape2007() -> nvim_oxi::Result<Dictionary> {
    let process_batch_fn = Function::from_fn(process_batch);
    let api = Dictionary::from_iter([("process_batch", process_batch_fn)]);
    Ok(api)
}

fn process_batch(input: String) -> bool {
    print!("{}", input);
    let mut _lexer = Token::lexer(&input);
    true
}

#[oxi::test]
fn process_batch_succeeds_base_case() {
    let result = process_batch("".to_string());
    assert_eq!(result, true);
}

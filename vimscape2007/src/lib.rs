use crate::{skills::Skills, token::Token};
use logos::Logos;
use nvim_oxi::{self as oxi, print, Dictionary, Function};

mod skills;
mod token;

#[nvim_oxi::plugin]
fn vimscape2007() -> nvim_oxi::Result<Dictionary> {
    let process_batch_fn = Function::from_fn(process_batch);
    let api = Dictionary::from_iter([("process_batch", process_batch_fn)]);
    Ok(api)
}

fn process_batch(input: String) -> bool {
    print!("{}", input);
    let mut lexer = Token::lexer(&input);
    let mut skills: Vec<Skills> = Vec::new();

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::MoveVerticalBasic(modifier)) => {
                let base_experience = 1;
                let experience = modifier * base_experience;
                skills.push(Skills::VerticalNavigation(experience))
            }
            Ok(Token::MoveHorizontalBasic(modifier)) => {
                let base_experience = 1;
                let experience = modifier * base_experience;
                skills.push(Skills::HorizontalNavigation(experience))
            }
            Ok(Token::MoveVerticalChunk) => {
                let base_experience = 10;
                skills.push(Skills::VerticalNavigation(base_experience))
            }
            Ok(Token::MoveHorizontalChunk(modifier)) => {
                let base_experience = 5;
                let experience = modifier * base_experience;
                skills.push(Skills::HorizontalNavigation(experience))
            }
            Ok(Token::JumpToHorizontal) => {}
            Ok(Token::JumpToLineNumber(line_number)) => {}
            Ok(Token::JumpToVertical) => {}
            Ok(Token::JumpFromContext) => {}
            Ok(Token::CameraMovement) => {}
            Ok(Token::WindowManagement) => {}
            Ok(Token::TextManipulationBasic(modifier)) => {}
            Ok(Token::TextManipulationAdvanced) => {}
            Ok(Token::YankPaste) => {}
            Ok(Token::UndoRedo) => {}
            Ok(Token::DotRepeat) => {}
            Ok(Token::CommandSearch(was_command_escaped)) => {}
            Ok(Token::DeleteText(modifier)) => {}
            Ok(Token::HelpPage(was_command_escaped)) => {}
            Ok(Token::SaveFile(was_command_escaped)) => {}
            Ok(Token::SaveFile(was_command_escaped)) => {}
            _ => {
                println!("Failed  to parse: {}", lexer.slice());
                continue;
            }
        }
    }

    true
}

#[oxi::test]
fn process_batch_succeeds_base_case() {
    let result = process_batch("".to_string());
    assert_eq!(result, true);
}

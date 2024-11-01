use std::collections::HashMap;

use crate::{db::create_tables, skills::Skills, token::Token};
use logos::Logos;
use nvim_oxi::{self as oxi, print, Dictionary, Function};

mod db;
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
    let mut skills: HashMap<String, i32> = HashMap::new();

    while let Some(token) = lexer.next() {
        println!("Got token {:?}", token);
        if let Some(result) = parse_action_into_skill(token) {
            println!("Parsed {},into {:?} skill", lexer.slice(), result);
            let skill_str = result.to_str();
            let new_exp = result.get_exp_from_skill();
            match skills.get(&*skill_str) {
                Some(total_exp) => skills.insert(skill_str, new_exp + total_exp),
                None => skills.insert(skill_str, new_exp),
            };
        } else {
            println!("Failed to parse: {}", lexer.slice());
        }
    }

    println!("Finished parsing, final skills: {:?}", skills);

    let _ = create_tables();

    // iter over hash map, write each to table

    true
}

fn parse_action_into_skill(token: Result<Token, ()>) -> Option<Skills> {
    match token {
        Ok(Token::MoveVerticalBasic(modifier)) => {
            let base_experience = 1;
            let experience = modifier * base_experience;
            return Some(Skills::VerticalNavigation(experience));
        }
        Ok(Token::MoveHorizontalBasic(modifier)) => {
            let base_experience = 1;
            let experience = modifier * base_experience;
            return Some(Skills::HorizontalNavigation(experience));
        }
        Ok(Token::MoveVerticalChunk) => {
            let base_experience = 10;
            return Some(Skills::VerticalNavigation(base_experience));
        }
        Ok(Token::MoveHorizontalChunk(modifier)) => {
            let base_experience = 5;
            let experience = modifier * base_experience;
            return Some(Skills::HorizontalNavigation(experience));
        }
        Ok(Token::JumpToHorizontal) => {
            let base_experience = 10;
            return Some(Skills::HorizontalNavigation(base_experience));
        }
        Ok(Token::JumpToLineNumber(_line_number)) => {
            let base_experience = 10;
            return Some(Skills::VerticalNavigation(base_experience));
        }
        Ok(Token::JumpToVertical) => {
            let base_experience = 10;
            return Some(Skills::VerticalNavigation(base_experience));
        }
        Ok(Token::JumpFromContext) => {
            let base_experience = 10;
            return Some(Skills::CodeFlow(base_experience));
        }
        Ok(Token::CameraMovement) => {
            let base_experience = 10;
            return Some(Skills::CameraMovement(base_experience));
        }
        Ok(Token::WindowManagement) => {
            let base_experience = 10;
            return Some(Skills::WindowManagement(base_experience));
        }
        Ok(Token::TextManipulationBasic(modifier)) => {
            let base_experience = 1;
            let experience = base_experience * modifier;
            return Some(Skills::TextManipulation(experience));
        }
        Ok(Token::TextManipulationAdvanced) => {
            let base_experience = 10;
            return Some(Skills::TextManipulation(base_experience));
        }
        Ok(Token::YankPaste) => {
            let base_experience = 10;
            return Some(Skills::Clipboard(base_experience));
        }
        Ok(Token::UndoRedo) => {
            let base_experience = 10;
            return Some(Skills::Clipboard(base_experience));
        }
        Ok(Token::DotRepeat) => {
            let base_experience = 10;
            return Some(Skills::Finesse(base_experience));
        }
        Ok(Token::CommandSearch(was_command_escaped)) => {
            let base_experience = if was_command_escaped { 1 } else { 10 };
            return Some(Skills::Search(base_experience));
        }
        Ok(Token::DeleteText(modifier)) => {
            let base_experience = 1;
            let experience = base_experience * modifier;
            return Some(Skills::TextManipulation(experience));
        }
        Ok(Token::HelpPage(was_command_escaped)) => {
            let base_experience = if was_command_escaped { 1 } else { 10 };
            return Some(Skills::Knowledge(base_experience));
        }
        Ok(Token::SaveFile(was_command_escaped)) => {
            let base_experience = if was_command_escaped { 1 } else { 10 };
            return Some(Skills::Saving(base_experience));
        }
        _ => {
            return None;
        }
    }
}

#[oxi::test]
fn process_batch_succeeds_base_case() {
    let result = process_batch("".to_string());
    assert_eq!(result, true);
}

#[oxi::test]
fn process_batch_prints_tokens_test() {
    let result = process_batch("jk3ldw:wgg".to_string());
    assert_eq!(result, true);
}

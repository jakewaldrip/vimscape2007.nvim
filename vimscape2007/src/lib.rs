use std::collections::HashMap;

use crate::{db::create_tables, db::write_results_to_table, skills::Skills, token::Token};
use logos::Logos;
use nvim_oxi::conversion::{Error as ConversionError, FromObject, ToObject};
use nvim_oxi::serde::{Deserializer, Serializer};
use nvim_oxi::{self as oxi, lua, print, Dictionary, Function, Object};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

mod db;
mod skills;
mod token;

#[derive(Serialize, Deserialize, Debug)]
struct UserData {
    skill_name: String,
    total_exp: i32,
    level: i32,
}

impl FromObject for UserData {
    fn from_object(obj: Object) -> Result<Self, ConversionError> {
        Self::deserialize(Deserializer::new(obj)).map_err(Into::into)
    }
}

impl ToObject for UserData {
    fn to_object(self) -> Result<Object, ConversionError> {
        self.serialize(Serializer::new()).map_err(Into::into)
    }
}

impl lua::Poppable for UserData {
    unsafe fn pop(lstate: *mut lua::ffi::lua_State) -> Result<Self, lua::Error> {
        let obj = Object::pop(lstate)?;
        Self::from_object(obj).map_err(lua::Error::pop_error_from_err::<Self, _>)
    }
}

impl lua::Pushable for UserData {
    unsafe fn push(self, lstate: *mut lua::ffi::lua_State) -> Result<std::ffi::c_int, lua::Error> {
        self.to_object()
            .map_err(lua::Error::push_error_from_err::<Self, _>)?
            .push(lstate)
    }
}

#[nvim_oxi::plugin]
fn vimscape2007() -> nvim_oxi::Result<Dictionary> {
    let process_batch_fn = Function::from_fn(process_batch);
    let get_user_data_fn = Function::from_fn(get_user_data);
    let api = Dictionary::from_iter([
        ("process_batch", Object::from(process_batch_fn)),
        ("get_user_data", Object::from(get_user_data_fn)),
    ]);
    Ok(api)
}

fn get_user_data(_: String) -> Vec<UserData> {
    let user_data = vec![
        UserData {
            skill_name: "jimbo".to_owned(),
            total_exp: 100,
            level: 32,
        },
        UserData {
            skill_name: "billy".to_owned(),
            total_exp: 327,
            level: 61,
        },
    ];

    return user_data;
}

fn process_batch(input: String) -> bool {
    print!("Input: {}", input);
    println!("Input: {}", input);
    println!("Input Length: {}", input.len());
    let mut lexer = Token::lexer(&input);
    let mut skills: HashMap<String, i32> = HashMap::new();

    while let Some(token) = lexer.next() {
        println!("Parsed token: {:?}", token);
        if let Some(result) = parse_action_into_skill(token) {
            println!("Parsed text: {} into skill {:?}", lexer.slice(), result);
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

    let conn = match Connection::open("test.db") {
        Ok(conn) => conn,
        Err(_) => {
            println!("Failed to connect to database");
            return false;
        }
    };

    create_tables(&conn);
    write_results_to_table(&conn, skills);
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
    let result = process_batch(
        r#"jk3l:w|enter|hd33ww:<C-U>call<Space>matchit#Match_wrapper('',1,'n')|enter|m'zvzzz:h test<Esc>jj:help test|enter|<C-W>s<C-W>v3""3puU<C-R>3w.3w/testsearch|enter|/testsearch2<Esc>hjkl"#
            .to_string(),
    );
    assert_eq!(result, true);
}

#[oxi::test]
fn get_user_data_base_case() {
    let result = get_user_data("".to_string());
    println!("result {:?}", result);
    assert_eq!(1, 1);
}

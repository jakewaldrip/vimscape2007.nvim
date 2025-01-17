use std::collections::HashMap;

use logos::Logos;
use rusqlite::Connection;

use crate::{
    db::{create_tables, get_skill_data, write_results_to_table},
    parse_utils::parse_action_into_skill,
    skill_data::SkillData,
    token::Token,
};

pub fn process_batch(input: String) -> bool {
    let mut lexer = Token::lexer(&input);
    let mut skills: HashMap<String, i32> = HashMap::new();

    while let Some(token) = lexer.next() {
        if let Some(result) = parse_action_into_skill(token) {
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

pub fn get_user_data(_: String) -> Vec<String> {
    let conn = match Connection::open("test.db") {
        Ok(conn) => conn,
        Err(_) => {
            println!("Failed to connect to database");
            return Vec::new();
        }
    };

    let mut display_strings: Vec<String> = Vec::new();
    let skill_data = get_skill_data(&conn).expect("Failed to connect to database");
    for data in skill_data {
        // TODO - find separator values from dotfiles and push here
        display_strings.push(data.format_skill_data());
    }
    display_strings
}

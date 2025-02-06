use std::{collections::HashMap, path::Path};

use logos::Logos;
use rusqlite::Connection;

use crate::{
    db::{create_tables, get_skill_data, write_results_to_table},
    parse_utils::parse_action_into_skill,
    skill_data::format_skill_data,
    token::Token,
};

pub fn process_batch((input, db_path): (String, String)) -> bool {
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

    let conn = match Connection::open(Path::new(&db_path).join("test.db")) {
        Ok(conn) => conn,
        Err(_) => {
            println!("Failed to connect to database");
            return false;
        }
    };

    write_results_to_table(&conn, skills);
    true
}

pub fn get_user_data((col_len, db_path): (i32, String)) -> Vec<String> {
    let conn = match Connection::open(Path::new(&db_path).join("test.db")) {
        Ok(conn) => conn,
        Err(_) => {
            println!("Failed to connect to database");
            return Vec::new();
        }
    };

    let skill_data = get_skill_data(&conn).expect("Failed to connect to database");
    let display_strings: Vec<String> = format_skill_data(&skill_data, col_len);
    display_strings
}

pub fn setup_tables(db_path: String) {
    let conn = match Connection::open(Path::new(&db_path).join("test.db")) {
        Ok(conn) => conn,
        Err(_) => {
            println!("Failed to connect to database");
            return ();
        }
    };

    create_tables(&conn);
}

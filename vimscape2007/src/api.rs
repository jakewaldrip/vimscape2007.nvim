use std::collections::HashMap;

use logos::Logos;
use rusqlite::Connection;

use crate::{
    db::{create_tables, write_results_to_table},
    parse_utils::parse_action_into_skill,
    skill_data::SkillData,
    token::Token,
};

pub fn process_batch(input: String) -> bool {
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

pub fn get_user_data(_: String) -> Vec<SkillData> {
    let user_data = vec![
        SkillData {
            skill_name: "jimbo".to_owned(),
            total_exp: 100,
            level: 32,
        },
        SkillData {
            skill_name: "billy".to_owned(),
            total_exp: 327,
            level: 61,
        },
    ];

    return user_data;
}

use std::{collections::HashMap, path::Path};

use rusqlite::Connection;

use crate::{
    db::{
        create_tables, get_skill_data, get_skill_details_from_db, write_exp_to_table,
        write_levels_to_table,
    },
    levels::{get_levels_diff, get_updated_levels, notify_level_ups},
    skill_data::{format_skill_data, format_skill_details},
    token::Token,
};

pub fn process_batch((input, db_path): (String, String)) -> bool {
    let skills: HashMap<String, i32> = HashMap::new();

    // while let Some(token) = lexer.next() {
    //     if let Some(result) = parse_action_into_skill(token) {
    //         let skill_str = result.to_str();
    //         let new_exp = result.get_exp_from_skill();
    //         match skills.get(&*skill_str) {
    //             Some(total_exp) => skills.insert(skill_str, new_exp + total_exp),
    //             None => skills.insert(skill_str, new_exp),
    //         };
    //     } else {
    //         println!("Failed to parse: {}", lexer.slice());
    //     }
    // }

    let conn = match Connection::open(Path::new(&db_path).join("teste.db")) {
        Ok(conn) => conn,
        Err(_) => {
            println!("Failed to connect to database");
            return false;
        }
    };

    let skill_data = get_skill_data(&conn).expect("Failed to query for skill data");
    let updated_levels = get_updated_levels(&skill_data, &skills);
    let levels_diff = get_levels_diff(&skill_data, &updated_levels);

    write_levels_to_table(&conn, &levels_diff);
    write_exp_to_table(&conn, skills);
    notify_level_ups(&levels_diff);

    true
}

pub fn get_user_data((col_len, db_path): (i32, String)) -> Vec<String> {
    let conn = match Connection::open(Path::new(&db_path).join("teste.db")) {
        Ok(conn) => conn,
        Err(_) => {
            println!("Failed to connect to database");
            return Vec::new();
        }
    };

    let skill_data = get_skill_data(&conn).expect("Failed to query for skill data");
    let display_strings: Vec<String> = format_skill_data(&skill_data, col_len);
    display_strings
}

pub fn setup_tables(db_path: String) {
    let conn = match Connection::open(Path::new(&db_path).join("teste.db")) {
        Ok(conn) => conn,
        Err(_) => {
            println!("Failed to connect to database");
            return;
        }
    };

    create_tables(&conn);
}

pub fn get_skill_details((c_word, db_path): (String, String)) -> Vec<String> {
    let conn = match Connection::open(Path::new(&db_path).join("teste.db")) {
        Ok(conn) => conn,
        Err(_) => {
            println!("Failed to connect to database");
            return Vec::new();
        }
    };

    let skill_data_vec =
        get_skill_details_from_db(&conn, &c_word).expect("Failed to get skills from database");
    if let Some(skill_data) = skill_data_vec.first() {
        format_skill_details(skill_data)
    } else {
        Vec::new()
    }
}

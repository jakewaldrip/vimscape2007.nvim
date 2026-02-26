use std::collections::HashMap;

use rusqlite::Connection;

use crate::{
    db::{
        create_tables, get_skill_data, get_skill_details_from_db, write_exp_to_table_tx,
        write_levels_to_table_tx,
    },
    levels::{get_levels_diff, get_updated_levels, notify_level_ups},
    lexer::Lexer,
    parse_utils::parse_action_into_skill,
    skill_data::{format_skill_data, format_skill_details},
};

pub fn process_batch((input, db_path): (String, String)) -> bool {
    let mut lexer = Lexer::new(&input);
    let mut skills: HashMap<String, i32> = HashMap::new();

    while let Some(token) = lexer.next_token() {
        if let Some(result) = parse_action_into_skill(&token) {
            let skill_str = result.to_str();
            let new_exp = result.get_exp_from_skill();
            match skills.get(&*skill_str) {
                Some(total_exp) => skills.insert(skill_str, new_exp + total_exp),
                None => skills.insert(skill_str, new_exp),
            };
        }
    }

    let Ok(conn) = Connection::open(&db_path) else {
        println!("Failed to connect to database");
        return false;
    };

    let skill_data = get_skill_data(&conn);
    if skill_data.is_empty() {
        eprintln!("[vimscape] No skill data found in database");
        return false;
    }

    let updated_levels = get_updated_levels(&skill_data, &skills);
    let levels_diff = get_levels_diff(&skill_data, &updated_levels);

    // Single transaction for all writes to ensure atomicity
    let tx = match conn.unchecked_transaction() {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("[vimscape] Transaction start failed: {e}");
            return false;
        }
    };

    // Write both levels and XP within the same transaction
    if !write_levels_to_table_tx(&tx, &levels_diff) {
        return false;
    }
    if !write_exp_to_table_tx(&tx, skills) {
        return false;
    }

    if let Err(e) = tx.commit() {
        eprintln!("[vimscape] Commit failed: {e}");
        return false;
    }

    // Notifications happen after successful commit
    notify_level_ups(&levels_diff);

    true
}

pub fn get_user_data((col_len, db_path): (i32, String)) -> Vec<String> {
    let Ok(conn) = Connection::open(&db_path) else {
        println!("Failed to connect to database");
        return Vec::new();
    };

    let skill_data = get_skill_data(&conn);
    format_skill_data(&skill_data, col_len)
}

#[allow(clippy::needless_pass_by_value)]
pub fn setup_tables(db_path: String) {
    let Ok(conn) = Connection::open(&db_path) else {
        println!("Failed to connect to database");
        return;
    };

    create_tables(&conn);
}

pub fn get_skill_details((c_word, db_path): (String, String)) -> Vec<String> {
    let Ok(conn) = Connection::open(&db_path) else {
        println!("Failed to connect to database");
        return Vec::new();
    };

    let skill_data_vec = get_skill_details_from_db(&conn, &c_word);
    if let Some(skill_data) = skill_data_vec.first() {
        format_skill_details(skill_data)
    } else {
        Vec::new()
    }
}

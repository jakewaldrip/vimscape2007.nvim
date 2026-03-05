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
    token::Token,
    token_log,
};

#[allow(clippy::needless_pass_by_value)]
pub fn enable_token_log(db_path: String) {
    token_log::enable(&db_path);
}

/// Remove consecutive duplicate `CameraMovement` tokens.
///
/// `vim.on_key` can fire twice for multi-character commands like `zz`/`zt`/`zb`,
/// producing duplicate tokens in the batch. This removes every second consecutive
/// `CameraMovement` to compensate.
fn dedup_tokens(tokens: &mut Vec<Token>) {
    let mut i = 1;
    while i < tokens.len() {
        if tokens[i] == Token::CameraMovement && tokens[i - 1] == Token::CameraMovement {
            tokens.remove(i);
            i += 1; // skip past the survivor to avoid re-matching
        } else {
            i += 1;
        }
    }
}

pub fn process_batch((input, db_path): (String, String)) -> bool {
    let mut lexer = Lexer::new(&input);
    let mut skills: HashMap<String, i32> = HashMap::new();
    let logging = token_log::is_enabled();

    if logging {
        token_log::log_batch(&input);
    }

    // Collect all tokens, then dedup before processing
    let mut tokens = Vec::new();
    while let Some(token) = lexer.next_token() {
        if logging {
            token_log::log_token(&token);
        }
        tokens.push(token);
    }

    dedup_tokens(&mut tokens);

    for token in &tokens {
        if let Some(result) = parse_action_into_skill(token) {
            let skill_str = result.to_str();
            let new_exp = result.get_exp_from_skill();
            match skills.get(&*skill_str) {
                Some(total_exp) => skills.insert(skill_str, new_exp + total_exp),
                None => skills.insert(skill_str, new_exp),
            };
        }
    }

    let Ok(conn) = Connection::open(&db_path) else {
        eprintln!("[vimscape] Failed to connect to database");
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

    notify_level_ups(&levels_diff);

    true
}

pub fn get_user_data((col_len, db_path): (i32, String)) -> Vec<String> {
    let Ok(conn) = Connection::open(&db_path) else {
        eprintln!("[vimscape] Failed to connect to database");
        return Vec::new();
    };

    let skill_data = get_skill_data(&conn);
    format_skill_data(&skill_data, col_len)
}

#[allow(clippy::needless_pass_by_value)]
pub fn setup_tables(db_path: String) {
    let Ok(conn) = Connection::open(&db_path) else {
        eprintln!("[vimscape] Failed to connect to database");
        return;
    };

    create_tables(&conn);
}

pub fn get_skill_details((c_word, db_path): (String, String)) -> Vec<String> {
    let Ok(conn) = Connection::open(&db_path) else {
        eprintln!("[vimscape] Failed to connect to database");
        return Vec::new();
    };

    let skill_data_vec = get_skill_details_from_db(&conn, &c_word);
    if let Some(skill_data) = skill_data_vec.first() {
        format_skill_details(skill_data)
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    /// Helper: lex input string into tokens, then run dedup
    fn lex_and_dedup(input: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        while let Some(token) = lexer.next_token() {
            tokens.push(token);
        }
        dedup_tokens(&mut tokens);
        tokens
    }

    /// Helper: count occurrences of CameraMovement in a token vec
    fn count_camera(tokens: &[Token]) -> usize {
        tokens
            .iter()
            .filter(|t| **t == Token::CameraMovement)
            .count()
    }

    #[test]
    fn test_dedup_single_zz() {
        // One zz command, no duplication — should remain 1
        let tokens = lex_and_dedup("zz");
        assert_eq!(count_camera(&tokens), 1);
    }

    #[test]
    fn test_dedup_duplicated_zz() {
        // zz duplicated by vim.on_key: zzzz → 2 CameraMovement → dedup to 1
        let tokens = lex_and_dedup("zzzz");
        assert_eq!(count_camera(&tokens), 1);
    }

    #[test]
    fn test_dedup_duplicated_zt() {
        // zt duplicated: ztzt → 2 CameraMovement → dedup to 1
        let tokens = lex_and_dedup("ztzt");
        assert_eq!(count_camera(&tokens), 1);
    }

    #[test]
    fn test_dedup_duplicated_zb() {
        // zb duplicated: zbzb → 2 CameraMovement → dedup to 1
        let tokens = lex_and_dedup("zbzb");
        assert_eq!(count_camera(&tokens), 1);
    }

    #[test]
    fn test_dedup_all_three_duplicated() {
        // zz, zt, zb each duplicated: zzzzztztzbzb → 6 CameraMovement → dedup to 3
        let tokens = lex_and_dedup("zzzzztztzbzb");
        assert_eq!(count_camera(&tokens), 3);
    }

    #[test]
    fn test_dedup_non_consecutive_camera() {
        // zz, then j, then zz — not consecutive, both kept
        let tokens = lex_and_dedup("zzjzz");
        assert_eq!(count_camera(&tokens), 2);
    }

    #[test]
    fn test_dedup_mixed_with_duplicated() {
        // Duplicated zz, then j, then duplicated zt
        let tokens = lex_and_dedup("zzzzjztzt");
        assert_eq!(count_camera(&tokens), 2);
        assert_eq!(tokens.len(), 3); // CM, MoveVerticalBasic, CM
    }

    #[test]
    fn test_dedup_preserves_other_tokens() {
        // No camera movements, just normal commands — nothing should change
        let tokens = lex_and_dedup("jjkk5w");
        assert_eq!(count_camera(&tokens), 0);
        assert_eq!(tokens.len(), 5);
    }

    #[test]
    fn test_dedup_empty() {
        let tokens = lex_and_dedup("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_dedup_three_consecutive_camera() {
        // 3 consecutive CameraMovement: first pair deduped, third survives
        // This simulates: one real zz (duplicated to zzzz) + one real zt (not duplicated)
        // Input "zzzzzt" → lexer produces CM, CM, CM → dedup removes second → CM, CM
        let tokens = lex_and_dedup("zzzzzt");
        assert_eq!(count_camera(&tokens), 2);
    }
}

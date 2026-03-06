use std::collections::HashMap;

use nvim_oxi::{
    Dictionary,
    api::{notify, types::LogLevel},
};
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

/// Notify the user of an error via Neovim's notification system.
/// Falls back to stderr if the Neovim API is unavailable.
fn notify_error(msg: &str) {
    let opts = Dictionary::new();
    if let Err(e) = notify(msg, LogLevel::Error, &opts) {
        eprintln!("[vimscape] {msg} (notify failed: {e:?})");
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn enable_token_log(db_path: String) {
    token_log::enable(&db_path);
}

/// Strip echoed leader-key sequences from the raw input.
///
/// Neovim's `vim.on_key` replays the expansion of custom `<Space>`-leader
/// mappings, producing a duplicated suffix in the batch string. For example,
/// pressing `<Space>sf` yields `|space|sf<Space>sf` — the real keystrokes
/// followed by a literal `<Space>` echo of the same characters.
///
/// This function finds every occurrence of `|space|{chars}<Space>{same chars}`
/// and removes the entire sequence. The binding suffix (`{chars}`) must be
/// short alphanumeric characters — this prevents false matches against
/// unrelated `|space|` / `<Space>` pairs separated by other commands.
fn strip_leader_echoes(input: &str) -> String {
    const PIPE_SPACE: &str = "|space|";
    const ANGLE_SPACE: &str = "<Space>";
    const MAX_BINDING_LEN: usize = 10;

    let mut result = String::with_capacity(input.len());
    let mut remaining = input;

    while let Some(pipe_pos) = remaining.find(PIPE_SPACE) {
        // Copy everything before this |space| verbatim (but don't copy |space| yet)
        result.push_str(&remaining[..pipe_pos]);
        remaining = &remaining[pipe_pos + PIPE_SPACE.len()..];

        // Extract the binding suffix: the short alphanumeric chars immediately
        // after |space| that form the leader keybinding (e.g., "sf" in |space|sf)
        let binding_len = remaining
            .chars()
            .take(MAX_BINDING_LEN)
            .take_while(char::is_ascii_alphanumeric)
            .count();

        if binding_len > 0 {
            let binding = &remaining[..binding_len];
            let after_binding = &remaining[binding_len..];

            // Check if <Space>{binding} immediately follows the binding chars
            let echo = format!("{ANGLE_SPACE}{binding}");
            if let Some(after_echo) = after_binding.strip_prefix(echo.as_str()) {
                // Leader echo confirmed — drop the entire |space|{chars}<Space>{chars}
                remaining = after_echo;
                continue;
            }
        }

        // Not a leader echo — put |space| back and continue
        result.push_str(PIPE_SPACE);
    }

    // Append any remaining input after the last match (or all of it if no match)
    result.push_str(remaining);
    result
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
    let input = strip_leader_echoes(&input);
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
        notify_error("[vimscape] Failed to connect to database");
        return false;
    };

    let skill_data = get_skill_data(&conn);
    if skill_data.is_empty() {
        notify_error("[vimscape] No skill data found in database");
        return false;
    }

    let updated_levels = get_updated_levels(&skill_data, &skills);
    let levels_diff = get_levels_diff(&skill_data, &updated_levels);

    // Single transaction for all writes to ensure atomicity
    let tx = match conn.unchecked_transaction() {
        Ok(tx) => tx,
        Err(e) => {
            notify_error(&format!("[vimscape] Transaction start failed: {e}"));
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
        notify_error(&format!("[vimscape] Commit failed: {e}"));
        return false;
    }

    notify_level_ups(&levels_diff);

    true
}

pub fn get_user_data((col_len, db_path): (i32, String)) -> Vec<String> {
    let Ok(conn) = Connection::open(&db_path) else {
        notify_error("[vimscape] Failed to connect to database");
        return Vec::new();
    };

    let skill_data = get_skill_data(&conn);
    format_skill_data(&skill_data, col_len)
}

#[allow(clippy::needless_pass_by_value)]
pub fn setup_tables(db_path: String) {
    let Ok(conn) = Connection::open(&db_path) else {
        notify_error("[vimscape] Failed to connect to database");
        return;
    };

    create_tables(&conn);
}

pub fn get_skill_details((c_word, db_path): (String, String)) -> Vec<String> {
    let Ok(conn) = Connection::open(&db_path) else {
        notify_error("[vimscape] Failed to connect to database");
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

    /// Helper: count occurrences of `CameraMovement` in a token vec
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

    // --- strip_leader_echoes tests ---

    #[test]
    fn test_strip_leader_basic() {
        // <Space>sf echoed as |space|sf<Space>sf → entire sequence removed
        let input = "|space|sf<Space>sf";
        assert_eq!(strip_leader_echoes(input), "");
    }

    #[test]
    fn test_strip_leader_single_char() {
        // Single-char leader binding: <Space>e → |space|e<Space>e
        let input = "|space|e<Space>e";
        assert_eq!(strip_leader_echoes(input), "");
    }

    #[test]
    fn test_strip_leader_longer_binding() {
        // Longer binding: <Space>abc → |space|abc<Space>abc
        let input = "|space|abc<Space>abc";
        assert_eq!(strip_leader_echoes(input), "");
    }

    #[test]
    fn test_strip_leader_with_surrounding_keys() {
        // Leader echo embedded within other keystrokes
        let input = "jjk|space|sf<Space>sfhhl";
        assert_eq!(strip_leader_echoes(input), "jjkhhl");
    }

    #[test]
    fn test_strip_leader_multiple_echoes() {
        // Two separate leader echoes in one batch
        let input = "|space|sf<Space>sfjj|space|ab<Space>ab";
        assert_eq!(strip_leader_echoes(input), "jj");
    }

    #[test]
    fn test_strip_leader_no_echo() {
        // |space| without a matching <Space> echo — left untouched
        let input = "|space|sfjjkk";
        assert_eq!(strip_leader_echoes(input), "|space|sfjjkk");
    }

    #[test]
    fn test_strip_leader_no_space_at_all() {
        // No |space| in input — returned unchanged
        let input = "jjkkhh";
        assert_eq!(strip_leader_echoes(input), "jjkkhh");
    }

    #[test]
    fn test_strip_leader_empty_input() {
        assert_eq!(strip_leader_echoes(""), "");
    }

    #[test]
    fn test_strip_leader_mismatch_not_stripped() {
        // <Space> is present but the chars after it don't match — no stripping
        let input = "|space|sf<Space>xy";
        assert_eq!(strip_leader_echoes(input), "|space|sf<Space>xy");
    }

    #[test]
    fn test_strip_leader_later_space_not_consumed_by_earlier_mismatch() {
        // Regression: multiple |space| where the first has no <Space> echo but
        // a later one does. The mismatch branch must not consume the <Space>
        // so the correct |space| can still match against it.
        let input = ":w|enter|<C-D><C-D><C-U><C-U>yyyy110yyGG|space|ss|space|s|escape|x|space|sw<Space>sw|escape|:q|escape|:Vimsacpe|backspace||backspace||backspace||backspace|cape|space|flush|enter|";
        let expected = ":w|enter|<C-D><C-D><C-U><C-U>yyyy110yyGG|space|ss|space|s|escape|x|escape|:q|escape|:Vimsacpe|backspace||backspace||backspace||backspace|cape|space|flush|enter|";
        assert_eq!(strip_leader_echoes(input), expected);
    }

    #[test]
    fn test_strip_leader_preserves_pipe_space_in_commands() {
        // |space| inside a command (e.g., :Vimscape|space|toggle) — no <Space> echo,
        // so it passes through untouched
        let input = ":Vimscape|space|toggle|enter|";
        assert_eq!(strip_leader_echoes(input), ":Vimscape|space|toggle|enter|");
    }
}

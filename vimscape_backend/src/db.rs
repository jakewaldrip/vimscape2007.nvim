use std::collections::HashMap;

use rusqlite::{params, Connection, Transaction};

use crate::{skill_data::SkillData, skills::Skills};

/// Retrieves all skill data from the database.
/// Returns an empty vector on error.
pub fn get_skill_data(conn: &Connection) -> Vec<SkillData> {
    let mut statement = match conn.prepare("SELECT name, exp, level FROM skills") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[vimscape] Query prepare failed: {e}");
            return Vec::new();
        }
    };

    let skill_data_iter = match statement.query_map([], |row| {
        Ok(SkillData {
            skill_name: row.get(0)?,
            total_exp: row.get(1)?,
            level: row.get(2)?,
        })
    }) {
        Ok(iter) => iter,
        Err(e) => {
            eprintln!("[vimscape] Query failed: {e}");
            return Vec::new();
        }
    };

    skill_data_iter.filter_map(|r| r.ok()).collect()
}

/// Creates the skills table and populates it with all skill types.
/// Returns true on success, false on failure.
pub fn create_tables(conn: &Connection) -> bool {
    if !create_skills_table(conn) {
        return false;
    }
    populate_skills_enum_table(conn)
}

fn create_skills_table(conn: &Connection) -> bool {
    if let Err(e) = conn.execute(
        "CREATE TABLE IF NOT EXISTS skills (
          id INTEGER PRIMARY KEY,
          name TEXT NOT NULL UNIQUE,
          exp INTEGER NOT NULL DEFAULT 0,
          level INTEGER NOT NULL DEFAULT 1
         )",
        (),
    ) {
        eprintln!("[vimscape] Create table failed: {e}");
        return false;
    }
    true
}

fn populate_skills_enum_table(conn: &Connection) -> bool {
    for (i, skill) in Skills::to_str_vec().iter().enumerate() {
        if let Err(e) = conn.execute(
            "INSERT OR IGNORE INTO skills (id, name) VALUES (?1, ?2)",
            params![i, skill],
        ) {
            eprintln!("[vimscape] Insert skill {skill} failed: {e}");
            // Continue with other skills
        }
    }
    true
}

/// Writes XP updates to the database.
/// Returns true on success, false on failure.
pub fn write_exp_to_table(conn: &Connection, skills: HashMap<String, i32>) -> bool {
    for (key, exp) in skills {
        if let Err(e) = conn.execute(
            "UPDATE skills SET exp = exp + ?1 WHERE name = ?2",
            params![exp, key],
        ) {
            eprintln!("[vimscape] Update XP failed for {key}: {e}");
            // Continue with other skills rather than aborting
        }
    }
    true
}

/// Writes level updates to the database.
/// Returns true on success, false on failure.
pub fn write_levels_to_table(conn: &Connection, levels_diff: &HashMap<String, i32>) -> bool {
    for (key, level) in levels_diff {
        if let Err(e) = conn.execute(
            "UPDATE skills SET level = ?1 WHERE name = ?2",
            params![level, key],
        ) {
            eprintln!("[vimscape] Update level failed for {key}: {e}");
            // Continue with other skills rather than aborting
        }
    }
    true
}

/// Retrieves skill data for a specific skill by name.
/// Returns an empty vector on error.
pub fn get_skill_details_from_db(conn: &Connection, skill_name: &str) -> Vec<SkillData> {
    let mut statement = match conn.prepare("SELECT name, exp, level FROM skills WHERE name = ?1") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[vimscape] Query prepare failed for skill {skill_name}: {e}");
            return Vec::new();
        }
    };

    let skill_data_iter = match statement.query_map(params![skill_name], |row| {
        Ok(SkillData {
            skill_name: row.get(0)?,
            total_exp: row.get(1)?,
            level: row.get(2)?,
        })
    }) {
        Ok(iter) => iter,
        Err(e) => {
            eprintln!("[vimscape] Query failed for skill {skill_name}: {e}");
            return Vec::new();
        }
    };

    skill_data_iter.filter_map(|r| r.ok()).collect()
}

/// Writes XP updates within an existing transaction.
/// Uses prepare_cached for efficiency in loops.
/// Returns true on success, false on failure.
pub fn write_exp_to_table_tx(tx: &Transaction, skills: HashMap<String, i32>) -> bool {
    let mut stmt = match tx.prepare_cached("UPDATE skills SET exp = exp + ?1 WHERE name = ?2") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[vimscape] Prepare failed: {e}");
            return false;
        }
    };

    for (key, exp) in skills {
        if let Err(e) = stmt.execute(params![exp, key]) {
            eprintln!("[vimscape] Update XP failed for {key}: {e}");
            // Continue with other skills rather than aborting
        }
    }

    true
}

/// Writes level updates within an existing transaction.
/// Uses prepare_cached for efficiency in loops.
/// Returns true on success, false on failure.
pub fn write_levels_to_table_tx(tx: &Transaction, levels_diff: &HashMap<String, i32>) -> bool {
    let mut stmt = match tx.prepare_cached("UPDATE skills SET level = ?1 WHERE name = ?2") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[vimscape] Prepare failed: {e}");
            return false;
        }
    };

    for (key, level) in levels_diff {
        if let Err(e) = stmt.execute(params![level, key]) {
            eprintln!("[vimscape] Update level failed for {key}: {e}");
            // Continue with other skills rather than aborting
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
        create_tables(&conn);
        conn
    }

    #[test]
    fn test_create_tables_creates_skills_table() {
        let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
        assert!(create_tables(&conn));

        // Verify table exists with expected columns
        let mut stmt = conn
            .prepare("SELECT name, exp, level FROM skills LIMIT 1")
            .expect("Skills table should exist with correct columns");
        let _ = stmt.query([]).expect("Query should succeed");
    }

    #[test]
    fn test_create_tables_populates_all_skills() {
        let conn = setup_test_db();
        let skills = get_skill_data(&conn);

        // Should have all skills from the Skills enum
        let expected_skills = Skills::to_str_vec();
        assert_eq!(
            skills.len(),
            expected_skills.len(),
            "Should have all skills populated"
        );

        for skill in &skills {
            assert!(
                expected_skills.iter().any(|s| *s == skill.skill_name),
                "Skill {} should be in expected list",
                skill.skill_name
            );
            assert_eq!(skill.total_exp, 0, "Initial XP should be 0");
            assert_eq!(skill.level, 1, "Initial level should be 1");
        }
    }

    #[test]
    fn test_get_skill_data_returns_all_skills() {
        let conn = setup_test_db();
        let skills = get_skill_data(&conn);

        assert!(!skills.is_empty(), "Should return populated skills");
        for skill in &skills {
            assert!(
                !skill.skill_name.is_empty(),
                "Skill name should not be empty"
            );
        }
    }

    #[test]
    fn test_get_skill_data_returns_empty_on_missing_table() {
        let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
        // Don't create tables
        let skills = get_skill_data(&conn);
        assert!(
            skills.is_empty(),
            "Should return empty vec when table doesn't exist"
        );
    }

    #[test]
    fn test_write_exp_to_table_updates_xp() {
        let conn = setup_test_db();

        let mut updates = HashMap::new();
        updates.insert("VerticalNavigation".to_string(), 100);
        updates.insert("TextManipulation".to_string(), 50);

        assert!(write_exp_to_table(&conn, updates));

        let skills = get_skill_data(&conn);
        let nav_skill = skills.iter().find(|s| s.skill_name == "VerticalNavigation");
        let edit_skill = skills.iter().find(|s| s.skill_name == "TextManipulation");

        assert_eq!(nav_skill.map(|s| s.total_exp), Some(100));
        assert_eq!(edit_skill.map(|s| s.total_exp), Some(50));
    }

    #[test]
    fn test_write_exp_to_table_accumulates_xp() {
        let conn = setup_test_db();

        let mut updates1 = HashMap::new();
        updates1.insert("VerticalNavigation".to_string(), 100);
        write_exp_to_table(&conn, updates1);

        let mut updates2 = HashMap::new();
        updates2.insert("VerticalNavigation".to_string(), 50);
        write_exp_to_table(&conn, updates2);

        let skills = get_skill_data(&conn);
        let nav_skill = skills.iter().find(|s| s.skill_name == "VerticalNavigation");
        assert_eq!(
            nav_skill.map(|s| s.total_exp),
            Some(150),
            "XP should accumulate"
        );
    }

    #[test]
    fn test_write_levels_to_table_updates_level() {
        let conn = setup_test_db();

        let mut updates = HashMap::new();
        updates.insert("VerticalNavigation".to_string(), 5);

        assert!(write_levels_to_table(&conn, &updates));

        let skills = get_skill_data(&conn);
        let nav_skill = skills.iter().find(|s| s.skill_name == "VerticalNavigation");
        assert_eq!(nav_skill.map(|s| s.level), Some(5));
    }

    #[test]
    fn test_get_skill_details_from_db_returns_single_skill() {
        let conn = setup_test_db();

        // Add some XP first
        let mut updates = HashMap::new();
        updates.insert("VerticalNavigation".to_string(), 200);
        write_exp_to_table(&conn, updates);

        let details = get_skill_details_from_db(&conn, "VerticalNavigation");
        assert_eq!(details.len(), 1, "Should return exactly one skill");
        assert_eq!(details[0].skill_name, "VerticalNavigation");
        assert_eq!(details[0].total_exp, 200);
    }

    #[test]
    fn test_get_skill_details_from_db_returns_empty_for_unknown_skill() {
        let conn = setup_test_db();
        let details = get_skill_details_from_db(&conn, "NonexistentSkill");
        assert!(details.is_empty(), "Should return empty for unknown skill");
    }

    #[test]
    fn test_write_exp_to_table_tx_within_transaction() {
        let mut conn = setup_test_db();

        let tx = conn.transaction().expect("Failed to start transaction");

        let mut updates = HashMap::new();
        updates.insert("VerticalNavigation".to_string(), 100);
        updates.insert("TextManipulation".to_string(), 75);

        assert!(write_exp_to_table_tx(&tx, updates));
        tx.commit().expect("Failed to commit transaction");

        let skills = get_skill_data(&conn);
        let nav_skill = skills.iter().find(|s| s.skill_name == "VerticalNavigation");
        let edit_skill = skills.iter().find(|s| s.skill_name == "TextManipulation");

        assert_eq!(nav_skill.map(|s| s.total_exp), Some(100));
        assert_eq!(edit_skill.map(|s| s.total_exp), Some(75));
    }

    #[test]
    fn test_write_levels_to_table_tx_within_transaction() {
        let mut conn = setup_test_db();

        let tx = conn.transaction().expect("Failed to start transaction");

        let mut updates = HashMap::new();
        updates.insert("VerticalNavigation".to_string(), 10);

        assert!(write_levels_to_table_tx(&tx, &updates));
        tx.commit().expect("Failed to commit transaction");

        let skills = get_skill_data(&conn);
        let nav_skill = skills.iter().find(|s| s.skill_name == "VerticalNavigation");
        assert_eq!(nav_skill.map(|s| s.level), Some(10));
    }

    #[test]
    fn test_transaction_rollback_on_drop() {
        let mut conn = setup_test_db();

        {
            let tx = conn.transaction().expect("Failed to start transaction");

            let mut updates = HashMap::new();
            updates.insert("VerticalNavigation".to_string(), 500);
            write_exp_to_table_tx(&tx, updates);

            // Don't commit - transaction should rollback on drop
        }

        let skills = get_skill_data(&conn);
        let nav_skill = skills.iter().find(|s| s.skill_name == "VerticalNavigation");
        assert_eq!(
            nav_skill.map(|s| s.total_exp),
            Some(0),
            "XP should not be updated after rollback"
        );
    }

    #[test]
    fn test_transaction_atomicity_both_writes() {
        let mut conn = setup_test_db();

        let tx = conn.transaction().expect("Failed to start transaction");

        // Write both XP and levels in same transaction
        let mut xp_updates = HashMap::new();
        xp_updates.insert("VerticalNavigation".to_string(), 1000);

        let mut level_updates = HashMap::new();
        level_updates.insert("VerticalNavigation".to_string(), 5);

        assert!(write_exp_to_table_tx(&tx, xp_updates));
        assert!(write_levels_to_table_tx(&tx, &level_updates));

        tx.commit().expect("Failed to commit transaction");

        let skills = get_skill_data(&conn);
        let nav_skill = skills.iter().find(|s| s.skill_name == "VerticalNavigation");

        assert_eq!(nav_skill.map(|s| s.total_exp), Some(1000));
        assert_eq!(nav_skill.map(|s| s.level), Some(5));
    }

    #[test]
    fn test_write_exp_handles_invalid_skill_gracefully() {
        let conn = setup_test_db();

        let mut updates = HashMap::new();
        updates.insert("InvalidSkillName".to_string(), 100);
        updates.insert("VerticalNavigation".to_string(), 50);

        // Should still return true and update valid skills
        assert!(write_exp_to_table(&conn, updates));

        let skills = get_skill_data(&conn);
        let nav_skill = skills.iter().find(|s| s.skill_name == "VerticalNavigation");
        assert_eq!(
            nav_skill.map(|s| s.total_exp),
            Some(50),
            "Valid skill should still be updated"
        );
    }

    #[test]
    fn test_create_tables_is_idempotent() {
        let conn = Connection::open_in_memory().expect("Failed to create in-memory database");

        // Create tables twice
        assert!(create_tables(&conn));
        assert!(create_tables(&conn));

        // Should still have all skills with correct initial values
        let skills = get_skill_data(&conn);
        let expected_count = Skills::to_str_vec().len();
        assert_eq!(
            skills.len(),
            expected_count,
            "Should have correct number of skills after double creation"
        );
    }
}

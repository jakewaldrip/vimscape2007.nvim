use std::collections::HashMap;

use rusqlite::{params, Connection, Error};

use crate::{skill_data::SkillData, skills::Skills};

pub fn get_skill_data(conn: &Connection) -> Result<Vec<SkillData>, Error> {
    let mut statement = conn.prepare("SELECT name, exp FROM skills")?;
    let skill_data_iter = statement.query_map([], |row| {
        Ok(SkillData {
            skill_name: row.get(0)?,
            total_exp: row.get(1)?,
            level: 15,
        })
    })?;

    skill_data_iter.collect()
}

pub fn create_tables(conn: &Connection) -> () {
    create_skills_table(&conn);
    populate_skills_enum_table(&conn);
}

pub fn write_results_to_table(conn: &Connection, skills: HashMap<String, i32>) -> () {
    for (key, exp) in skills {
        let _ = conn.execute(
            "update skills set exp = exp + ?1 where name = ?2",
            params![exp, key],
        );
    }
}

fn create_skills_table(conn: &Connection) -> () {
    let _ = conn.execute(
        "create table if not exists skills (
          id integer primary key,
          name text not null unique,
          exp integer not null default 0
         )",
        (),
    );
}

fn populate_skills_enum_table(conn: &Connection) -> () {
    for (i, skill) in Skills::to_str_vec().iter().enumerate() {
        let _ = conn.execute(
            "insert or ignore into skills (id, name) values (?1, ?2)",
            params![i, skill],
        );
    }
}

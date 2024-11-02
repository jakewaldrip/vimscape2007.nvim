use std::collections::HashMap;

use rusqlite::{params, Connection, Result};

use crate::skills::Skills;

pub fn create_tables(conn: &Connection) -> () {
    println!("Creating tables");
    create_skills_table(&conn);
    populate_skills_enum_table(&conn);
}

pub fn write_results_to_table(conn: &Connection, skills: HashMap<String, i32>) -> () {
    todo!();
}

fn create_skills_table(conn: &Connection) -> () {
    let _ = conn.execute(
        "create table if not exists skills (
          id integer primary key,
          name text not null unique
         )",
        (),
    );
}

fn populate_skills_enum_table(conn: &Connection) -> () {
    for (i, skill) in Skills::to_str_vec().iter().enumerate() {
        let _ = conn.execute(
            "replace into skills (id, name) values (?1, ?2)",
            params![i, skill],
        );
    }
}

use rusqlite::{params, Connection, Result};

use crate::skills::Skills;

pub fn create_tables() -> Result<()> {
    println!("Creating tables");
    let conn = Connection::open("vimscape.db")?;

    let _ = conn.execute(
        "create table if not exists skills (
          id integer primary key,
          name text not null unique
         )",
        (),
    );

    return populate_skills_enum_table(conn);
}

fn populate_skills_enum_table(conn: Connection) -> Result<()> {
    for (i, skill) in Skills::to_str_vec().iter().enumerate() {
        let _ = conn.execute(
            "replace into skills (id, name) values (?1, ?2)",
            params![i, skill],
        );
    }
    Ok(())
}

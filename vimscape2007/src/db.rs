use rusqlite::{Connection, Result};

pub fn create_tables() -> Result<()> {
    println!("creating tables");
    let conn = Connection::open("vimscape.db")?;

    conn.execute(
        "create table if not exists test (
          id integer primary key,
          name text not null
         )",
        (),
    );

    Ok(())
}

use std::process;

use anyhow::Result;
use rusqlite::Connection;
use spdlog::error;

use super::schema::{GROUP_SCHEMA, NODE_SCHEMA, RUNTIME_SCHEMA};

pub fn test_and_create_node_table(conn: &Connection) -> Result<()> {
    conn.execute(NODE_SCHEMA, [])?;
    Ok(())
}

pub fn test_and_create_group_table(conn: &Connection) -> Result<()> {
    conn.execute(GROUP_SCHEMA, [])?;
    Ok(())
}

pub fn test_and_create_runtime_table(conn: &Connection) -> Result<()> {
    conn.execute(RUNTIME_SCHEMA, [])?;
    Ok(())
}

pub fn count_table(connection: &Connection, name: &str) -> Result<usize> {
    let sql = format!("SELECT COUNT(*) FROM {}", name);
    let mut statement = connection.prepare(&sql)?;
    let mut rows = statement.query([])?;
    let size;
    match rows.next()? {
        Some(row) => {
            size = row.get(0)?;
        }
        None => {
            error!("SQLite Count Error");
            process::exit(1);
        }
    }
    Ok(size)
}
#[cfg(test)]
mod tests {
    use anyhow::Result;
    use rusqlite::{params, Connection};

    use crate::tools::dbtools::count_table;
    #[test]
    fn test_count_table() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE testtable (
                  id              INTEGER PRIMARY KEY,
                  name            TEXT NOT NULL
                  )",
            [],
        )?;
        assert_eq!(0, count_table(&conn, "testtable")?);
        for i in 1..15 {
            println!("{}!", i);
            let name = format!("test name {}", i);
            conn.execute("INSERT INTO testtable (name) VALUES (?1)", params![name])?;
            assert_eq!(i, count_table(&conn, "testtable")?);
        }
        Ok(())
    }
}

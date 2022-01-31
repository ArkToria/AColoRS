use anyhow::Result;
use rusqlite::Connection;

pub fn create_node_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE nodes(
                    ID INTEGER PRIMARY KEY AUTOINCREMENT,
                    Name TEXT NOT NULL,
                    GroupID INTEGER NOT NULL,
                    GroupName TEXT NOT NULL,
                    RoutingID INTEGER NOT NULL,
                    RoutingName TEXT NOT NULL,
                    Protocol INTEGER NOT NULL,
                    Address TEXT NOT NULL,
                    Port INTEGER NOT NULL,
                    Password TEXT,
                    Raw TEXT NOT NULL,
                    URL TEXT NOT NULL,
                    Latency INTEGER,
                    Upload INT64,
                    Download INT64,
                    CreatedAt INT64 NOT NULL,
                    ModifiedAt INT64 NOT NULL
                    )",
        [],
    )?;
    Ok(())
}

pub fn create_group_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE groups(
                    ID INTEGER PRIMARY KEY AUTOINCREMENT,
                    Name TEXT UNIQUE NOT NULL,
                    IsSubscription BOOLEAN NOT NULL,
                    Type INTEGER NOT NULL,
                    Url TEXT,
                    CycleTime INTEGER,
                    CreatedAt INT64 NOT NULL,
                    ModifiedAt INT64 NOT NULL
                    )",
        [],
    )?;
    Ok(())
}

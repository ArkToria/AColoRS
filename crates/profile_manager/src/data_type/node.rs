use anyhow::Result;
use rusqlite::Connection;
use utils::time::get_current_time;

#[derive(Debug)]
pub struct Node {
    pub id: i32,
    pub name: String,
    pub group_id: i32,
    pub group_name: String,
    pub routing_id: i32,
    pub routing_name: String,
    pub protocol: i32,
    pub address: String,
    pub port: u16,
    pub password: String,
    pub raw: String,
    pub url: String,
    pub latency: i32,
    pub upload: i64,
    pub download: i64,
    pub create_at: i64,
    pub modified_at: i64,
}

impl Node {
    pub fn update_modified_at(&mut self) {
        self.modified_at = get_current_time() as i64;
    }

    pub fn update_create_at(&mut self) {
        self.create_at = get_current_time() as i64;
    }
}

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

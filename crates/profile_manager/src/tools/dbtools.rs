use std::sync::Arc;

use sqlx::Database;
use tokio::sync::Mutex;

type DatabaseDriver = sqlx::Sqlite;
type SharedConnection = Arc<Mutex<<DatabaseDriver as Database>::Connection>>;

use super::schema::{GROUP_SCHEMA, NODE_SCHEMA, RUNTIME_SCHEMA};

pub async fn test_and_create_node_table(conn: &SharedConnection) -> sqlx::Result<()> {
    let query = sqlx::query(NODE_SCHEMA);
    let conn_mut = &mut *conn.lock().await;
    query.execute(conn_mut).await?;
    Ok(())
}

pub async fn test_and_create_group_table(conn: &SharedConnection) -> sqlx::Result<()> {
    let query = sqlx::query(GROUP_SCHEMA);
    let conn_mut = &mut *conn.lock().await;
    query.execute(conn_mut).await?;
    Ok(())
}

pub async fn test_and_create_runtime_table(conn: &SharedConnection) -> sqlx::Result<()> {
    let query = sqlx::query(RUNTIME_SCHEMA);
    let conn_mut = &mut *conn.lock().await;
    query.execute(conn_mut).await?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use anyhow::Result;
    use sqlx::{Connection, SqliteConnection};

    #[test]
    fn test_sth() -> Result<()> {
        Ok(())
    }
    #[tokio::test]
    async fn test_count_table() -> Result<()> {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await?;
        sqlx::query(
            "CREATE TABLE testtable (
                  id              INTEGER PRIMARY KEY,
                  name            TEXT NOT NULL
                  )",
        )
        .execute(&mut conn)
        .await?;
        /*
        assert_eq!(0, count_table(&conn, "testtable")?);
        for i in 1..15 {
            println!("{}!", i);
            let name = format!("test name {}", i);
            conn.execute("INSERT INTO testtable (name) VALUES (?1)", params![name])?;
            assert_eq!(i, count_table(&conn, "testtable")?);
        }
        */
        Ok(())
    }
}

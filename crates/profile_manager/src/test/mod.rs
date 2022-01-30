#[cfg(test)]
mod tests {
    use crate::Group;
    use std::error::Error;

    use rusqlite::Connection;

    type Result<T> = ::std::result::Result<T, Box<dyn Error>>;

    #[test]
    fn it_works() -> Result<()> {
        let conn = Connection::open_in_memory()?;

        create_group_table(&conn)?;
        let group = Group {
            id: 0,
            name: "test group".to_string(),
            is_subscription: false,
            group_type: 0,
            url: "".to_string(),
            cycle_time: 0,
            create_at: 1637666014,
            modified_at: 1637666614,
        };

        assert_eq!(2 + 2, 4);
        Ok(())
    }

    fn create_test_table(conn: &Connection) -> Result<()> {
        create_group_table(conn)?;
        add_sample_groups(conn)?;

        Ok(())
    }

    fn add_sample_groups(conn: &Connection) -> Result<()> {
        Ok(())
    }

    fn create_group_table(conn: &Connection) -> Result<()> {
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
}

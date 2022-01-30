#[derive(Debug)]
struct Group {
    id: i32,
    name: String,
    is_subscription: bool,
    group_type: i32,
    url: String,
    cycle_time: i32,
    create_at: i64,
    modified_at: i64,
}
#[cfg(test)]
mod tests {
    use crate::Group;
    use std::error::Error;

    use rusqlite::Connection;

    #[test]
    fn it_works() -> Result<(), Box<dyn Error>> {
        let conn = Connection::open_in_memory()?;

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
}

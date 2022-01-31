#[cfg(test)]
mod tests {
    use std::error::Error;

    use profile_manager::{
        data_type::group::Group,
        dbtools::{create_group_table, create_node_table},
    };
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

        Ok(())
    }

    fn create_test_table(conn: &Connection) -> Result<()> {
        create_group_table(conn)?;
        create_node_table(conn)?;
        add_sample_groups(conn)?;
        // TODO: add nodes and more

        Ok(())
    }

    fn add_sample_groups(conn: &Connection) -> Result<()> {
        // TODO: insert some groups
        Ok(())
    }
}

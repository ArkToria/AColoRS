#[cfg(test)]
mod tests {
    use std::error::Error;

    use profile_manager::{
        data_type::group::GroupData,
        tools::dbtools::{test_and_create_group_table, test_and_create_node_table},
    };
    use rusqlite::Connection;

    type Result<T> = ::std::result::Result<T, Box<dyn Error>>;

    #[test]
    fn it_works() -> Result<()> {
        Ok(())
    }

    fn create_test_table(conn: &Connection) -> Result<()> {
        test_and_create_group_table(conn)?;
        test_and_create_node_table(conn)?;
        add_sample_groups(conn)?;
        // TODO: add nodes and more

        Ok(())
    }

    fn add_sample_groups(conn: &Connection) -> Result<()> {
        // TODO: insert some groups
        Ok(())
    }
}

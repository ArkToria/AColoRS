use std::rc::Rc;

use rusqlite::Connection;

use crate::data_type::group::Group;

use super::{
    group::GroupData,
    traits::{AColoRSListModel, HasTable, WithConnection},
};

#[derive(Debug)]
pub struct GroupList {
    connection: Rc<Connection>,
}

impl GroupList {
    pub fn new(connection: Rc<Connection>) -> GroupList {
        GroupList { connection }
    }
}

const GROUP_TABLE_NAME: &str = "groups";
impl HasTable for GroupList {
    fn has_table_name() -> &'static str {
        GROUP_TABLE_NAME
    }
}

impl WithConnection for GroupList {
    fn connection(&self) -> Rc<Connection> {
        self.connection.clone()
    }
}
impl AColoRSListModel<Group, GroupData> for GroupList {}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        data_type::group::tests::{compare_group, generate_test_group},
        tools::dbtools::{test_and_create_group_table, test_and_create_node_table},
    };
    use anyhow::Result;

    #[test]
    fn test_insert_into_group_and_query() -> Result<()> {
        let conn = Rc::new(Connection::open_in_memory()?);
        test_and_create_group_table(&conn)?;
        test_and_create_node_table(&conn)?;
        let mut group_list = GroupList::new(conn);
        for i in 1..15 {
            let group_data = generate_test_group(i);
            group_list.append(&group_data)?;
            let fetch_group = group_list.query(i as usize)?;
            println!("{:?}", fetch_group);
            assert!(compare_group(fetch_group.data(), &group_data));
        }
        Ok(())
    }
    #[test]
    fn test_update_group_and_query() -> Result<()> {
        let conn = Rc::new(Connection::open_in_memory()?);
        test_and_create_group_table(&conn)?;
        test_and_create_node_table(&conn)?;
        let mut group_list = GroupList::new(conn);
        for i in 1..15 {
            let group_data = generate_test_group(i);
            group_list.append(&group_data)?;
            let fetch_group = group_list.query(i as usize)?;
            println!("Before: {:?}", fetch_group);
            assert!(compare_group(fetch_group.data(), &group_data));

            let new_group = generate_test_group(i + 200);
            group_list.set(fetch_group.data().id as usize, &new_group)?;
            let fetch_group = group_list.query(i as usize)?;

            println!("After: {:?}", fetch_group);
            assert!(compare_group(fetch_group.data(), &new_group));
        }
        Ok(())
    }
}

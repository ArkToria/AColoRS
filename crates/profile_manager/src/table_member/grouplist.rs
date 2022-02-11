use std::rc::Rc;

use rusqlite::Connection;

use crate::table_member::group::Group;
use crate::tools::dbtools::test_and_create_node_table;
use crate::{data_type::group::*, tools::dbtools::test_and_create_group_table};

use super::traits::{AColoRSListModel, HasTable, WithConnection};

#[derive(Debug)]
pub struct GroupList {
    connection: Rc<Connection>,
}

impl GroupList {
    pub fn new(connection: Rc<Connection>) -> GroupList {
        test_and_create_group_table(&connection).unwrap();
        test_and_create_node_table(&connection).unwrap();
        GroupList { connection }
    }
    pub fn list_all_groups(&self) -> anyhow::Result<Vec<Group>> {
        let sql = "SELECT * FROM groups";
        let mut statement = self.connection.prepare(sql)?;
        let mut result: Vec<Group> = Vec::new();
        let mut rows = statement.query([])?;
        while let Some(row) = rows.next()? {
            let group_data = GroupData {
                id: row.get(0)?,
                name: row.get(1)?,
                is_subscription: row.get(2)?,
                group_type: row.get(3)?,
                url: row.get(4)?,
                cycle_time: row.get(5)?,
                create_at: row.get(6)?,
                modified_at: row.get(7)?,
            };
            result.push(Group::new(group_data, self.connection.clone()));
        }
        Ok(result)
    }

    /// For node query
    pub fn default_group(&self) -> Group {
        let data = GroupData::default();
        Group::new(data, self.connection.clone())
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

impl AColoRSListModel<Group, GroupData> for GroupList {
    fn append(&mut self, item: &GroupData) -> anyhow::Result<()> {
        let mut item = item.clone();

        item.update_create_at();
        item.update_modified_at();

        crate::tools::dbtools::insert_into_table::<Group, GroupData>(&self.connection(), &item)
    }

    fn set(&mut self, id: usize, item: &GroupData) -> anyhow::Result<()> {
        let mut item = item.clone();
        item.update_modified_at();

        crate::tools::dbtools::update_table::<Group, GroupData>(&self.connection(), id, &item)
    }

    fn remove(&mut self, id: usize) -> anyhow::Result<()> {
        let group = self.query(id)?;

        group.remove_all_nodes()?;
        crate::tools::dbtools::remove_from_table::<Group, GroupData>(&self.connection(), id)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        table_member::group::tests::{compare_group, generate_test_group},
        tools::dbtools::{test_and_create_group_table, test_and_create_node_table},
    };
    use anyhow::{anyhow, Result};

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

    #[test]
    fn test_remove_group_and_query() -> Result<()> {
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

            group_list.remove(fetch_group.data().id as usize)?;
            let fetch_group = group_list.query(i as usize);
            let error_expected = anyhow!("Group Not Found");

            if let Err(e) = fetch_group {
                assert_eq!(error_expected.to_string(), e.to_string());
            } else {
                panic!("No Errors when group removed");
            }
        }
        Ok(())
    }

    #[test]
    fn test_insert_into_group_and_list_all() -> Result<()> {
        let conn = Rc::new(Connection::open_in_memory()?);
        test_and_create_group_table(&conn)?;
        test_and_create_node_table(&conn)?;
        let mut group_list = GroupList::new(conn);
        let mut group_vec: Vec<Group> = Vec::new();
        for i in 1..15 {
            let group_data = generate_test_group(i);
            group_list.append(&group_data)?;
            let fetch_group = group_list.query(i as usize)?;
            group_vec.push(fetch_group.clone());
            assert!(compare_group(fetch_group.data(), &group_data));
        }
        let nodes = group_list.list_all_groups()?;
        for i in 0..14 {
            let q_group = group_vec[i].data();
            println!("{}: \n{:?}", i, q_group);
            let s_group = nodes[i].data();
            println!("{:?}\n", s_group);
            assert!(compare_group(group_vec[i].data(), nodes[i].data()));
        }
        Ok(())
    }
}

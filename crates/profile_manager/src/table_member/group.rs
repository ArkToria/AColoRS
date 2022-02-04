use std::rc::Rc;

use anyhow::anyhow;
use rusqlite::{params, Connection};

use crate::{
    data_type::{group::*, node::*},
    tools::dbtools::{insert_into_table, update_table},
};

use super::{
    node::Node,
    traits::{AColoRSListModel, AttachedToTable, HasTable, WithConnection},
};

#[derive(Debug, Clone)]
pub struct Group {
    data: GroupData,
    connection: Rc<Connection>,
}

impl Group {
    /// Get a reference to the group's data.
    pub fn data(&self) -> &GroupData {
        &self.data
    }

    pub fn list_all_nodes(&self) -> anyhow::Result<Vec<Node>> {
        let group_id = self.data().id;
        let sql = "SELECT * FROM nodes WHERE GroupID = ?";
        let mut statement = self.connection.prepare(&sql)?;
        let mut result: Vec<Node> = Vec::new();
        let mut rows = statement.query(&[&group_id])?;
        while let Some(row) = rows.next()? {
            let node_data = NodeData {
                id: row.get(0)?,
                name: row.get(1)?,
                group_id: row.get(2)?,
                group_name: row.get(3)?,
                routing_id: row.get(4)?,
                routing_name: row.get(5)?,
                protocol: row.get(6)?,
                address: row.get(7)?,
                port: row.get(8)?,
                password: row.get(9)?,
                raw: row.get(10)?,
                url: row.get(11)?,
                latency: row.get(12)?,
                upload: row.get(13)?,
                download: row.get(14)?,
                create_at: row.get(15)?,
                modified_at: row.get(16)?,
            };
            result.push(Node::new(node_data, self.connection.clone()));
        }
        Ok(result)
    }

    pub fn new(data: GroupData, connection: Rc<Connection>) -> Group {
        Group { data, connection }
    }

    pub fn to_data(self) -> GroupData {
        self.data
    }
}

const NODE_TABLE_NAME: &str = "nodes";

impl AttachedToTable<GroupData> for Group {
    fn attached_to_table_name() -> &'static str {
        GROUP_TABLE_NAME
    }
    fn field_names() -> &'static [&'static str] {
        GROUP_FIELD_NAMES
    }
    fn get_update_sql() -> &'static str {
        GROUP_UPDATE_SQL
    }
    fn get_insert_sql() -> &'static str {
        GROUP_INSERT_SQL
    }
    fn get_remove_sql() -> &'static str {
        GROUP_REMOVE_SQL
    }

    fn get_query_sql() -> &'static str {
        GROUP_QUERY_SQL
    }

    fn execute_statement(
        item_data: &GroupData,
        statement: &mut rusqlite::Statement,
    ) -> rusqlite::Result<usize> {
        statement.execute(params![
            item_data.name,
            item_data.is_subscription,
            item_data.group_type,
            item_data.url,
            item_data.cycle_time,
            item_data.create_at,
            item_data.modified_at,
        ])
    }
    fn execute_statement_with_id(
        item_data: &GroupData,
        id: usize,
        statement: &mut rusqlite::Statement,
    ) -> rusqlite::Result<usize> {
        statement.execute(params![
            item_data.name,
            item_data.is_subscription,
            item_data.group_type,
            item_data.url,
            item_data.cycle_time,
            item_data.create_at,
            item_data.modified_at,
            id,
        ])
    }

    fn query_map(
        connection: Rc<Connection>,
        statement: &mut rusqlite::Statement,
        id: usize,
    ) -> anyhow::Result<Group> {
        let iter = statement.query_map(&[&id], |row| {
            Ok(GroupData {
                id: row.get(0)?,
                name: row.get(1)?,
                is_subscription: row.get(2)?,
                group_type: row.get(3)?,
                url: row.get(4)?,
                cycle_time: row.get(5)?,
                create_at: row.get(6)?,
                modified_at: row.get(7)?,
            })
        })?;
        for data in iter {
            return Ok(Group {
                data: data?,
                connection,
            });
        }
        Err(anyhow!("Group Not Found"))
    }
}
impl HasTable for Group {
    fn has_table_name() -> &'static str {
        NODE_TABLE_NAME
    }
}

impl WithConnection for Group {
    fn connection(&self) -> Rc<Connection> {
        self.connection.clone()
    }
}

impl AColoRSListModel<Node, NodeData> for Group {
    fn append(&mut self, item: &NodeData) -> anyhow::Result<()> {
        let mut item = item.clone();
        item.group_id = self.data().id;
        insert_into_table::<Node, NodeData>(&self.connection(), &item)
    }
    fn set(&mut self, id: usize, item: &NodeData) -> anyhow::Result<()> {
        let mut item = item.clone();
        item.group_id = self.data().id;
        update_table::<Node, NodeData>(&self.connection(), id, &item)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{
        table_member::{
            grouplist::GroupList,
            node::tests::{compare_node, generate_test_node},
        },
        tools::dbtools::{test_and_create_group_table, test_and_create_node_table},
    };

    use super::*;
    use anyhow::Result;
    #[test]
    fn test_insert_into_node_and_query() -> Result<()> {
        let conn = Rc::new(Connection::open_in_memory()?);
        test_and_create_group_table(&conn)?;
        test_and_create_node_table(&conn)?;
        let mut group_list = GroupList::new(conn);
        group_list.append(&generate_test_group(1))?;
        group_list.append(&generate_test_group(2))?;
        group_list.append(&generate_test_group(3))?;
        let mut group = group_list.query(2)?;
        for i in 1..15 {
            let node_data = generate_test_node(i);
            group.append(&node_data)?;
            let fetch_node = group.query(i as usize)?;
            println!("{:?}", fetch_node);
            assert!(compare_node(fetch_node.data(), &node_data));
        }
        Ok(())
    }
    #[test]
    fn test_update_node_and_query() -> Result<()> {
        let conn = Rc::new(Connection::open_in_memory()?);
        test_and_create_group_table(&conn)?;
        test_and_create_node_table(&conn)?;
        let mut group_list = GroupList::new(conn);
        group_list.append(&generate_test_group(1))?;
        group_list.append(&generate_test_group(2))?;
        group_list.append(&generate_test_group(3))?;
        let mut group = group_list.query(2)?;
        for i in 1..15 {
            let node_data = generate_test_node(i);
            group.append(&node_data)?;
            let fetch_node = group.query(i as usize)?;
            assert!(compare_node(fetch_node.data(), &node_data));
            println!("Before: {:?}", fetch_node);

            let new_node = generate_test_node(i + 200);
            group.set(fetch_node.data().id as usize, &new_node)?;
            let fetch_node = group.query(i as usize)?;

            println!("After: {:?}", fetch_node);
            assert!(compare_node(fetch_node.data(), &new_node));
        }
        Ok(())
    }
    #[test]
    fn test_remove_node_and_query() -> Result<()> {
        let conn = Rc::new(Connection::open_in_memory()?);
        test_and_create_group_table(&conn)?;
        test_and_create_node_table(&conn)?;
        let mut group_list = GroupList::new(conn);
        group_list.append(&generate_test_group(1))?;
        group_list.append(&generate_test_group(2))?;
        group_list.append(&generate_test_group(3))?;
        let mut group = group_list.query(2)?;
        for i in 1..15 {
            let node_data = generate_test_node(i);
            group.append(&node_data)?;
            let fetch_node = group.query(i as usize)?;
            assert!(compare_node(fetch_node.data(), &node_data));
            println!("Before: {:?}", fetch_node);

            group.remove(fetch_node.data().id as usize)?;
            let fetch_node = group.query(i as usize);
            let error_expected = anyhow!("Node Not Found");

            if let Err(e) = fetch_node {
                assert_eq!(error_expected.to_string(), e.to_string());
            } else {
                panic!("No Errors when group removed");
            }
        }
        Ok(())
    }
    #[test]
    fn test_insert_into_node_and_list_all() -> Result<()> {
        let conn = Rc::new(Connection::open_in_memory()?);
        test_and_create_group_table(&conn)?;
        test_and_create_node_table(&conn)?;
        let mut group_list = GroupList::new(conn);
        group_list.append(&generate_test_group(1))?;
        group_list.append(&generate_test_group(2))?;
        group_list.append(&generate_test_group(3))?;
        let mut group = group_list.query(2)?;
        let mut node_list: Vec<Node> = Vec::new();
        for i in 1..15 {
            let node_data = generate_test_node(i);
            group.append(&node_data)?;
            let fetch_node = group.query(i as usize)?;
            node_list.push(fetch_node.clone());
            assert!(compare_node(fetch_node.data(), &node_data));
        }
        let nodes = group.list_all_nodes()?;
        for i in 0..14 {
            let q_node = node_list[i].data();
            println!("{}: \n{:?}", i, q_node);
            let s_node = nodes[i].data();
            println!("{:?}\n", s_node);
            assert!(compare_node(node_list[i].data(), nodes[i].data()));
        }
        Ok(())
    }
    pub fn compare_group(a: &GroupData, b: &GroupData) -> bool {
        let mut ac = a.clone();
        let mut bc = b.clone();
        ac.id = 0;
        bc.id = 0;
        ac == bc
    }
    pub fn generate_test_group(number: u16) -> GroupData {
        let test_string = format!("test{}", number);
        let mut result = GroupData {
            id: number as i32,
            name: format!("{} group", &test_string),
            is_subscription: false,
            group_type: 0,
            url: format!("https://localhost:{}", number),
            cycle_time: number as i32,
            create_at: 0,
            modified_at: 0,
        };
        result.update_create_at();
        result.update_modified_at();
        result
    }
}

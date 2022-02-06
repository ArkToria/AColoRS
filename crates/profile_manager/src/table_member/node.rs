use std::rc::Rc;

use anyhow::anyhow;
use rusqlite::{params, Connection};

use crate::data_type::node::*;

use super::traits::{AttachedToTable, WithConnection};

#[derive(Debug, Clone)]
pub struct Node {
    data: NodeData,
    connection: Rc<Connection>,
}

impl Node {
    /// Get a reference to the node's data.
    pub fn data(&self) -> &NodeData {
        &self.data
    }

    pub fn to_data(self) -> NodeData {
        self.data
    }

    pub fn new(data: NodeData, connection: Rc<Connection>) -> Node {
        Node { data, connection }
    }
}

impl AttachedToTable<NodeData> for Node {
    fn attached_to_table_name() -> &'static str {
        NODE_TABLE_NAME
    }
    fn field_names() -> &'static [&'static str] {
        NODE_FIELD_NAMES
    }

    fn get_update_sql() -> &'static str {
        NODE_UPDATE_SQL
    }
    fn get_insert_sql() -> &'static str {
        NODE_INSERT_SQL
    }

    fn get_remove_sql() -> &'static str {
        NODE_REMOVE_SQL
    }

    fn get_query_sql() -> &'static str {
        NODE_QUERY_SQL
    }

    fn execute_statement(
        item_data: &NodeData,
        statement: &mut rusqlite::Statement,
    ) -> rusqlite::Result<usize> {
        statement.execute(params![
            item_data.name,
            item_data.group_id,
            item_data.group_name,
            item_data.routing_id,
            item_data.routing_name,
            item_data.protocol,
            item_data.address,
            item_data.port,
            item_data.password,
            item_data.raw,
            item_data.url,
            item_data.latency,
            item_data.upload,
            item_data.download,
            item_data.create_at,
            item_data.modified_at,
        ])
    }
    fn execute_statement_with_id(
        item_data: &NodeData,
        id: usize,
        statement: &mut rusqlite::Statement,
    ) -> rusqlite::Result<usize> {
        statement.execute(params![
            item_data.name,
            item_data.group_id,
            item_data.group_name,
            item_data.routing_id,
            item_data.routing_name,
            item_data.protocol,
            item_data.address,
            item_data.port,
            item_data.password,
            item_data.raw,
            item_data.url,
            item_data.latency,
            item_data.upload,
            item_data.download,
            item_data.create_at,
            item_data.modified_at,
            id,
        ])
    }
    fn query_map(
        connection: Rc<Connection>,
        statement: &mut rusqlite::Statement,
        id: usize,
    ) -> anyhow::Result<Node> {
        let iter = statement.query_map(&[&id], |row| {
            Ok(NodeData {
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
            })
        })?;
        for data in iter {
            return Ok(Node {
                data: data?,
                connection,
            });
        }
        Err(anyhow!("Node Not Found"))
    }
}
impl WithConnection for Node {
    fn connection(&self) -> Rc<Connection> {
        self.connection.clone()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    pub fn compare_node(a: &NodeData, b: &NodeData) -> bool {
        let mut ac = a.clone();
        let mut bc = b.clone();
        ac.id = 0;
        bc.id = 0;
        ac.group_id = 0;
        bc.group_id = 0;
        ac.group_name = String::new();
        bc.group_name = String::new();
        ac == bc
    }
    pub fn generate_test_node(number: i32) -> NodeData {
        let test_string = format!("test{}", number);
        let test_address = format!("localhost:{}", number);
        let mut result = NodeData {
            id: number,
            name: format!("{} node", test_string),
            group_id: number,
            group_name: format!("{} group", test_string),
            routing_id: number,
            routing_name: format!("{} routing", test_string),
            protocol: number,
            address: test_address.clone(),
            port: number,
            password: test_string.clone(),
            raw: test_string.clone(),
            url: format!("https://{}", test_address),
            latency: 100 * number,
            upload: 200 * (number as i64),
            download: 300 * (number as i64),
            create_at: 0,
            modified_at: 0,
        };
        result.update_create_at();
        result.update_modified_at();
        result
    }
}

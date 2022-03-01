use std::sync::Arc;

use core_data::data_type::node::*;
use sqlx::SqliteConnection;
use tokio::sync::Mutex;

use super::traits::{AttachedToTable, WithConnection};

type Connection = Arc<Mutex<SqliteConnection>>;
#[derive(Debug, Clone)]
pub struct Node {
    data: NodeData,
    connection: Connection,
}

impl Node {
    /// Get a reference to the node's data.
    pub fn data(&self) -> &NodeData {
        &self.data
    }

    pub fn to_data(self) -> NodeData {
        self.data
    }

    pub fn new(data: NodeData, connection: Connection) -> Node {
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
}
impl WithConnection for Node {
    fn connection(&self) -> Connection {
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

        // TODO: routing
        ac.routing_id = 0;
        bc.routing_id = 0;
        ac.latency = -1;
        bc.latency = -1;
        ac.routing_name = String::new();
        bc.routing_name = String::new();

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

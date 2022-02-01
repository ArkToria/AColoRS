use std::rc::Rc;

use rusqlite::Connection;
use utils::time::get_current_time;

use super::withconnection::WithConnection;

const NODE_TABLE_NAME: &'static str = "nodes";
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
}

impl WithConnection for Node {
    fn table_name() -> String {
        NODE_TABLE_NAME.to_string()
    }
    fn connection(&self) -> Rc<Connection> {
        self.connection.clone()
    }
}

#[derive(Debug, Clone)]
pub struct NodeData {
    pub id: i32,
    pub name: String,
    pub group_id: i32,
    pub group_name: String,
    pub routing_id: i32,
    pub routing_name: String,
    pub protocol: i32,
    pub address: String,
    pub port: u16,
    pub password: String,
    pub raw: String,
    pub url: String,
    pub latency: i32,
    pub upload: i64,
    pub download: i64,
    pub create_at: i64,
    pub modified_at: i64,
}

impl NodeData {
    pub fn update_modified_at(&mut self) {
        self.modified_at = get_current_time() as i64;
    }

    pub fn update_create_at(&mut self) {
        self.create_at = get_current_time() as i64;
    }
}

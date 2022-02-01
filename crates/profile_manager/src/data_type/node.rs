use std::rc::Rc;

use rusqlite::{params, Connection};
use utils::time::get_current_time;

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
}

impl AttachedToTable for Node {
    fn attached_to_table_name() -> &'static str {
        NODE_TABLE_NAME
    }
    fn field_names() -> &'static [&'static str] {
        NODE_FIELD_NAMES
    }

    fn execute_statement(&self, statement: &mut rusqlite::Statement) -> rusqlite::Result<usize> {
        statement.execute(params![
            self.data.id,
            self.data.name,
            self.data.group_id,
            self.data.group_name,
            self.data.routing_id,
            self.data.routing_name,
            self.data.protocol,
            self.data.address,
            self.data.port,
            self.data.password,
            self.data.raw,
            self.data.url,
            self.data.latency,
            self.data.upload,
            self.data.download,
            self.data.create_at,
            self.data.modified_at,
        ])
    }
}
impl WithConnection for Node {
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
const NODE_TABLE_NAME: &str = "nodes";
const NODE_FIELD_NAMES: &[&str] = &[
    "Name",
    "GroupID",
    "GroupName",
    "RoutingID",
    "RoutingName",
    "Protocol",
    "Address",
    "Port",
    "Password",
    "Raw",
    "URL",
    "Latency",
    "Upload",
    "Download",
    "CreatedAt",
    "ModifiedAt",
];

impl NodeData {
    pub fn update_modified_at(&mut self) {
        self.modified_at = get_current_time() as i64;
    }

    pub fn update_create_at(&mut self) {
        self.create_at = get_current_time() as i64;
    }
}

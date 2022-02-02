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

impl AttachedToTable<NodeData> for Node {
    fn attached_to_table_name() -> &'static str {
        NODE_TABLE_NAME
    }
    fn field_names() -> &'static [&'static str] {
        NODE_FIELD_NAMES
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

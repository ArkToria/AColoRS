use std::rc::Rc;

use rusqlite::Connection;
use utils::time::get_current_time;

use super::{
    listmodel::AColoRSListModel,
    node::Node,
    withconnection::{AttachedToTable, WithConnection},
};

const GROUP_TABLE_NAME: &'static str = "nodes";
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
}
#[derive(Debug, Clone)]
pub struct GroupData {
    pub id: i32,
    pub name: String,
    pub is_subscription: bool,
    pub group_type: i32,
    pub url: String,
    pub cycle_time: i32,
    pub create_at: i64,
    pub modified_at: i64,
}

impl GroupData {
    pub fn update_modified_at(&mut self) {
        self.modified_at = get_current_time() as i64;
    }

    pub fn update_create_at(&mut self) {
        self.create_at = get_current_time() as i64;
    }
}

impl AttachedToTable for Group {
    fn table_name() -> String {
        GROUP_TABLE_NAME.to_string()
    }
}

impl WithConnection for Group {
    fn connection(&self) -> Rc<Connection> {
        self.connection.clone()
    }
}

impl AColoRSListModel<Node> for Group {}

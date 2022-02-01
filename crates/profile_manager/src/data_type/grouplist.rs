use std::rc::Rc;

use rusqlite::Connection;

use crate::data_type::group::Group;

use super::{
    listmodel::AColoRSListModel,
    withconnection::{AttachedToTable, WithConnection},
};

const GROUP_LIST_TABLE_NAME: &'static str = "groups";
#[derive(Debug)]
pub struct GroupList {
    connection: Rc<Connection>,
}

impl GroupList {
    pub fn new(connection: Rc<Connection>) -> GroupList {
        GroupList { connection }
    }
}

impl AttachedToTable for GroupList {
    fn table_name() -> String {
        GROUP_LIST_TABLE_NAME.to_string()
    }
}

impl WithConnection for GroupList {
    fn connection(&self) -> Rc<Connection> {
        self.connection.clone()
    }
}
impl AColoRSListModel<Group> for GroupList {}

use std::rc::Rc;

use rusqlite::Connection;

use crate::data_type::group::Group;

use super::traits::{AColoRSListModel, HasTable, WithConnection};

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
impl AColoRSListModel<Group> for GroupList {}

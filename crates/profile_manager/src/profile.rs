use std::rc::Rc;

use rusqlite::Connection;

use crate::table_member::{grouplist::GroupList, runtime::RuntimeValue};

#[derive(Debug)]
pub struct Profile {
    pub group_list: GroupList,
    pub runtime_value: RuntimeValue,
}

impl Profile {
    pub fn new(connection: Rc<Connection>) -> Profile {
        Profile {
            group_list: GroupList::new(connection.clone()),
            runtime_value: RuntimeValue::new(connection.clone()),
        }
    }
}

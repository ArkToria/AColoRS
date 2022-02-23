use std::rc::Rc;

use rusqlite::Connection;

use crate::table_member::{grouplist::GroupList, runtime::RuntimeValue};

#[derive(Debug)]
pub struct Profile {
    pub group_list: GroupList,
    pub runtime_value: RuntimeValue,
}

impl Profile {
    pub fn new(connection: Connection) -> rusqlite::Result<Profile> {
        let connection = Rc::new(connection);
        Ok(Profile {
            group_list: GroupList::create(connection.clone())?,
            runtime_value: RuntimeValue::create(connection)?,
        })
    }
}

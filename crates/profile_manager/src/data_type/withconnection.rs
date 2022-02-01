use std::rc::Rc;

use rusqlite::Connection;

pub trait WithConnection {
    fn connection(&self) -> Rc<Connection>;
}

pub trait AttachedToTable: WithConnection {
    fn table_name() -> String;
}

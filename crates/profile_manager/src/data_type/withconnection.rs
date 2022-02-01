use std::rc::Rc;

use rusqlite::Connection;

pub trait WithConnection {
    fn connection(&self) -> Rc<Connection>;
    fn table_name() -> String;
}

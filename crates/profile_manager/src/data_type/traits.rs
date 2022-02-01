use anyhow::Result;

use crate::tools::dbtools::count_table;

use std::rc::Rc;

use rusqlite::Connection;

pub trait AColoRSListModel<T: Clone>: AttachedToTable {
    fn size(&self) -> Result<usize> {
        count_table(&self.connection(), &Self::table_name())
    }
    fn append(&mut self, item: &T) -> Result<()> {
        todo!();
        //insert_into_table(&self.connection(), &Self::table_name(), item);
    }
    fn set(&mut self, index: usize, item: &T) -> Result<()> {
        todo!();
    }
    fn remove(&mut self, index: usize) -> Result<()> {
        todo!();
    }
    fn get(&self) -> T {
        todo!();
    }
}

pub trait WithConnection {
    fn connection(&self) -> Rc<Connection>;
}

pub trait AttachedToTable: WithConnection {
    fn table_name() -> String;
}

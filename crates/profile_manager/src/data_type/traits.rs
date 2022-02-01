use anyhow::Result;

use crate::tools::dbtools::{count_table, insert_into_table};

use std::rc::Rc;

use rusqlite::{Connection, Statement};

pub trait AColoRSListModel<T: Clone + AttachedToTable>: HasTable {
    fn size(&self) -> Result<usize> {
        count_table(&self.connection(), Self::has_table_name())
    }
    fn append(&mut self, item: &T) -> Result<()> {
        insert_into_table(&self.connection(), item)
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
    fn attached_to_table_name() -> &'static str;
    fn field_names() -> &'static [&'static str];
    fn execute_statement(&self, statement: &mut Statement) -> rusqlite::Result<usize>;
}

pub trait HasTable: WithConnection {
    fn has_table_name() -> &'static str;
}

use anyhow::Result;

use crate::tools::dbtools::{count_table, insert_into_table, update_table};

use std::rc::Rc;

use rusqlite::{Connection, Statement};

pub trait AColoRSListModel<T, D>: HasTable
where
    T: Clone + AttachedToTable<D>,
    D: Clone,
{
    fn size(&self) -> Result<usize> {
        count_table(&self.connection(), Self::has_table_name())
    }
    fn append(&mut self, item: &D) -> Result<()> {
        insert_into_table::<T, D>(&self.connection(), item)
    }
    fn set(&mut self, id: usize, item: &D) -> Result<()> {
        update_table::<T, D>(&self.connection(), id, item)
    }
    fn remove(&mut self, id: usize) -> Result<()> {
        remove_from_table(&self.connection(), id)
    }
    fn get(&self, id: usize) -> T {
        todo!();
    }
}

fn remove_from_table(connection: &Connection, id: usize) -> Result<(), anyhow::Error> {
    todo!()
}

pub trait WithConnection {
    fn connection(&self) -> Rc<Connection>;
}

pub trait AttachedToTable<D>: WithConnection
where
    D: Clone,
{
    fn attached_to_table_name() -> &'static str;
    fn field_names() -> &'static [&'static str];
    fn execute_statement(item_data: &D, statement: &mut Statement) -> rusqlite::Result<usize>;
    fn execute_statement_with_id(
        item_data: &D,
        id: usize,
        statement: &mut Statement,
    ) -> rusqlite::Result<usize>;
}

pub trait HasTable: WithConnection {
    fn has_table_name() -> &'static str;
}

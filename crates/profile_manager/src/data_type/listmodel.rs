use anyhow::Result;

use crate::tools::dbtools::count_table;

use super::withconnection::AttachedToTable;

pub trait AColoRSListModel<T: Clone>: AttachedToTable {
    fn size(&self) -> Result<usize> {
        count_table(&self.connection(), &Self::table_name())
    }
    fn append(&mut self, item: &T) -> Result<()> {
        todo!();
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

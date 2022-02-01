use std::process;

use anyhow::Result;
use spdlog::error;

use super::withconnection::WithConnection;

pub trait AColoRSListModel<T: Clone>: WithConnection {
    fn size(&self) -> Result<usize> {
        let sql = format!("SELECT COUNT(*) FROM {}", Self::table_name());
        let connection = self.connection();
        let mut statement = connection.prepare(&sql)?;
        let mut rows = statement.query([])?;
        let size;
        match rows.next()? {
            Some(row) => {
                size = row.get(0)?;
            }
            None => {
                error!("SQLite Count Error");
                process::exit(1);
            }
        }
        Ok(size)
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

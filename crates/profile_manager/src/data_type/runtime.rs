use anyhow::anyhow;
use std::rc::Rc;

use rusqlite::{params, Connection};

use super::traits::{AColoRSListModel, AttachedToTable, HasTable, WithConnection};

#[derive(Debug, Clone)]
pub struct RuntimeValue {
    connection: Rc<Connection>,
}
impl RuntimeValue {
    pub fn new(connection: Rc<Connection>) -> Self {
        RuntimeValue { connection }
    }
}
impl WithConnection for RuntimeValue {
    fn connection(&self) -> Rc<Connection> {
        self.connection.clone()
    }
}
const RUNTIME_TABLE_NAME: &str = "runtime";
impl HasTable for RuntimeValue {
    fn has_table_name() -> &'static str {
        RUNTIME_TABLE_NAME
    }
}
impl AColoRSListModel<Value, ValueData> for RuntimeValue {}

#[derive(Debug, Clone)]
pub struct Value {
    data: ValueData,
    connection: Rc<Connection>,
}

impl Value {
    /// Get a reference to the value's data.
    pub fn data(&self) -> &ValueData {
        &self.data
    }
}

#[derive(Debug, Clone)]
pub struct ValueData {
    pub id: i32,
    pub name: String,
    pub value_type: i32,
    pub value: String,
}

impl WithConnection for Value {
    fn connection(&self) -> Rc<Connection> {
        self.connection.clone()
    }
}
const RUNTIME_FIELD_NAMES: &[&str] = &["Name", "Type", "Value"];
impl AttachedToTable<ValueData> for Value {
    fn attached_to_table_name() -> &'static str {
        RUNTIME_TABLE_NAME
    }

    fn field_names() -> &'static [&'static str] {
        RUNTIME_FIELD_NAMES
    }

    fn execute_statement(
        item_data: &ValueData,
        statement: &mut rusqlite::Statement,
    ) -> rusqlite::Result<usize> {
        statement.execute(params![
            item_data.name,
            item_data.value_type,
            item_data.value,
        ])
    }

    fn execute_statement_with_id(
        item_data: &ValueData,
        id: usize,
        statement: &mut rusqlite::Statement,
    ) -> rusqlite::Result<usize> {
        statement.execute(params![
            item_data.name,
            item_data.value_type,
            item_data.value,
            id,
        ])
    }

    fn query_map(
        connection: Rc<Connection>,
        statement: &mut rusqlite::Statement,
        id: usize,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let iter = statement.query_map(&[&id], |row| {
            Ok(ValueData {
                id: row.get(0)?,
                name: row.get(1)?,
                value_type: row.get(2)?,
                value: row.get(3)?,
            })
        })?;
        for data in iter {
            match data {
                Ok(d) => {
                    return Ok(Value {
                        data: d,
                        connection,
                    })
                }
                Err(e) => return Err(anyhow!("{}", e)),
            }
        }
        Err(anyhow!("Group Not Found"))
    }
}

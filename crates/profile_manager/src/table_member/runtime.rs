use anyhow::anyhow;
use spdlog::error;
use std::rc::Rc;

use rusqlite::{params, Connection};

use super::traits::{AColoRSListModel, AttachedToTable, HasTable, WithConnection};
use crate::tools::dbtools::test_and_create_runtime_table;
use core_data::data_type::runtimevalue::*;

#[derive(Debug, Clone)]
pub struct RuntimeValue {
    connection: Rc<Connection>,
}
impl RuntimeValue {
    pub fn new(connection: Rc<Connection>) -> Self {
        if let Err(e) = test_and_create_runtime_table(&connection) {
            error!("{}", e);
        }
        RuntimeValue { connection }
    }
}
impl WithConnection for RuntimeValue {
    fn connection(&self) -> Rc<Connection> {
        self.connection.clone()
    }
}
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

impl WithConnection for Value {
    fn connection(&self) -> Rc<Connection> {
        self.connection.clone()
    }
}

impl AttachedToTable<ValueData> for Value {
    fn attached_to_table_name() -> &'static str {
        RUNTIME_TABLE_NAME
    }

    fn field_names() -> &'static [&'static str] {
        RUNTIME_FIELD_NAMES
    }

    fn get_update_sql() -> &'static str {
        RUNTIME_UPDATE_SQL
    }

    fn get_insert_sql() -> &'static str {
        RUNTIME_INSERT_SQL
    }

    fn get_remove_sql() -> &'static str {
        RUNTIME_REMOVE_SQL
    }

    fn get_query_sql() -> &'static str {
        RUNTIME_QUERY_SQL
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
        let mut iter = statement.query_map(&[&id], |row| {
            Ok(ValueData {
                id: row.get(0)?,
                name: row.get(1)?,
                value_type: row.get(2)?,
                value: row.get(3)?,
            })
        })?;
        if let Some(data) = iter.next() {
            return Ok(Value {
                data: data?,
                connection,
            });
        }
        Err(anyhow!("RuntimeValue Not Found"))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::tools::dbtools::test_and_create_runtime_table;
    use anyhow::{anyhow, Result};

    #[test]
    fn test_insert_into_runtime_value_and_query() -> Result<()> {
        let conn = Rc::new(Connection::open_in_memory()?);
        test_and_create_runtime_table(&conn)?;
        let mut runtime_value = RuntimeValue::new(conn);
        for i in 1..15 {
            let runtime_value_data = generate_test_runtime_value(i);
            runtime_value.append(&runtime_value_data)?;
            let fetch_runtime_value = runtime_value.query(i as usize)?;
            println!("{:?}", fetch_runtime_value);
            assert!(compare_runtime_value(
                fetch_runtime_value.data(),
                &runtime_value_data
            ));
        }
        Ok(())
    }
    #[test]
    fn test_update_runtime_value_and_query() -> Result<()> {
        let conn = Rc::new(Connection::open_in_memory()?);
        test_and_create_runtime_table(&conn)?;
        let mut runtime_value = RuntimeValue::new(conn);
        for i in 1..15 {
            let runtime_value_data = generate_test_runtime_value(i);
            runtime_value.append(&runtime_value_data)?;
            let fetch_runtime_value = runtime_value.query(i as usize)?;
            println!("Before: {:?}", fetch_runtime_value);
            assert!(compare_runtime_value(
                fetch_runtime_value.data(),
                &runtime_value_data
            ));

            let new_runtime_value = generate_test_runtime_value(i + 200);
            runtime_value.set(fetch_runtime_value.data().id as usize, &new_runtime_value)?;
            let fetch_runtime_value = runtime_value.query(i as usize)?;

            println!("After: {:?}", fetch_runtime_value);
            assert!(compare_runtime_value(
                fetch_runtime_value.data(),
                &new_runtime_value
            ));
        }
        Ok(())
    }

    #[test]
    fn test_remove_runtime_value_and_query() -> Result<()> {
        let conn = Rc::new(Connection::open_in_memory()?);
        test_and_create_runtime_table(&conn)?;
        let mut runtime_value = RuntimeValue::new(conn);
        for i in 1..15 {
            let runtime_value_data = generate_test_runtime_value(i);
            runtime_value.append(&runtime_value_data)?;
            let fetch_runtime_value = runtime_value.query(i as usize)?;
            println!("Before: {:?}", fetch_runtime_value);
            assert!(compare_runtime_value(
                fetch_runtime_value.data(),
                &runtime_value_data
            ));

            runtime_value.remove(fetch_runtime_value.data().id as usize)?;
            let fetch_runtime_value = runtime_value.query(i as usize);
            let error_expected = anyhow!("RuntimeValue Not Found");

            if let Err(e) = fetch_runtime_value {
                assert_eq!(error_expected.to_string(), e.to_string());
            } else {
                panic!("No Errors when runtime_value removed");
            }
        }
        Ok(())
    }
    pub fn compare_runtime_value(a: &ValueData, b: &ValueData) -> bool {
        let mut ac = a.clone();
        let mut bc = b.clone();
        ac.id = 0;
        bc.id = 0;
        ac == bc
    }
    pub fn generate_test_runtime_value(number: u16) -> ValueData {
        let test_string = format!("test{}", number);
        let result = ValueData {
            id: number as i32,
            name: format!("{} runtime_value", &test_string),
            value_type: number as i32,
            value: test_string,
        };
        result
    }
}

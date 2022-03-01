use futures::TryStreamExt;
use spdlog::error;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use sqlx::{Database, Row};

use super::traits::{AttachedToTable, HasTable, WithConnection};
use crate::tools::dbtools::test_and_create_runtime_table;
use core_data::data_type::runtimevalue::*;

type DatabaseDriver = sqlx::Sqlite;
type SharedConnection = Arc<Mutex<<DatabaseDriver as Database>::Connection>>;
#[derive(Debug)]
pub struct RuntimeValue {
    map: Mutex<HashMap<String, ValueData>>,
    connection: SharedConnection,
}
impl RuntimeValue {
    pub async fn create(connection: SharedConnection) -> sqlx::Result<Self> {
        if let Err(e) = test_and_create_runtime_table(&connection).await {
            error!("{}", e);
        }

        let mut map = HashMap::new();
        let conn_mut = &mut *connection.lock().await;
        let mut rows = sqlx::query("SELECT * FROM runtime;").fetch(conn_mut);
        while let Some(row) = rows.try_next().await? {
            let value_data = ValueData {
                id: row.try_get(0)?,
                name: row.try_get(1)?,
                value_type: row.try_get(2)?,
                value: row.try_get(3)?,
            };
            map.insert(value_data.name.clone(), value_data);
        }
        let map = Mutex::new(map);
        let connection = connection.clone();
        Ok(RuntimeValue { map, connection })
    }
    pub async fn set_by_key(&self, key: &str, value: String) -> sqlx::Result<()> {
        let map_mut = &mut *self.map.lock().await;
        match map_mut.get_mut(key) {
            Some(v) => {
                v.value = value;
            }
            None => {
                let value_data = ValueData {
                    id: 0,
                    name: key.to_string(),
                    value_type: 2,
                    value,
                };
                self.append(value_data.clone()).await?;
                map_mut.insert(value_data.name.clone(), value_data);

                return Ok(());
            }
        }

        let value = map_mut.get(key).unwrap().clone();

        self.set(value.id as i64, value).await?;

        Ok(())
    }
    pub async fn get_by_key(&self, key: &str) -> Option<String> {
        let map = &*self.map.lock().await;
        map.get(key).map(|v| v.value.clone())
    }

    async fn append(&self, item: ValueData) -> sqlx::Result<i64> {
        let query = sqlx::query(RUNTIME_INSERT_SQL)
            .bind(item.name)
            .bind(item.value_type)
            .bind(item.value);
        let conn_mut = &mut *self.connection.lock().await;

        let result = query.execute(conn_mut).await?.last_insert_rowid();
        Ok(result)
    }
    async fn set(&self, id: i64, item: ValueData) -> sqlx::Result<()> {
        let query = sqlx::query(RUNTIME_UPDATE_SQL)
            .bind(item.name)
            .bind(item.value_type)
            .bind(item.value)
            .bind(id);

        let conn_mut = &mut *self.connection.lock().await;

        query.execute(conn_mut).await?;
        Ok(())
    }
    pub async fn query(&self, id: i64) -> sqlx::Result<Value> {
        let query = sqlx::query(RUNTIME_QUERY_SQL).bind(id);
        let conn_mut = &mut *self.connection.lock().await;
        let mut rows = query.fetch(conn_mut);
        let data = match rows.try_next().await? {
            Some(row) => ValueData {
                id: row.try_get(0)?,
                name: row.try_get(1)?,
                value_type: row.try_get(2)?,
                value: row.try_get(3)?,
            },
            None => return Err(sqlx::Error::RowNotFound),
        };
        Ok(Value::new(data, self.connection.clone()))
    }
    pub async fn remove(&self, id: i64) -> sqlx::Result<()> {
        let query = sqlx::query(RUNTIME_REMOVE_SQL).bind(id);
        let conn_mut = &mut *self.connection.lock().await;

        query.execute(conn_mut).await?;
        Ok(())
    }
}
impl WithConnection for RuntimeValue {
    fn connection(&self) -> SharedConnection {
        self.connection.clone()
    }
}
impl HasTable for RuntimeValue {
    fn has_table_name() -> &'static str {
        RUNTIME_TABLE_NAME
    }
}

#[derive(Debug, Clone)]
pub struct Value {
    data: ValueData,
    connection: SharedConnection,
}

impl Value {
    pub fn new(data: ValueData, connection: SharedConnection) -> Value {
        Value { data, connection }
    }
    /// Get a reference to the value's data.
    pub fn data(&self) -> &ValueData {
        &self.data
    }
}

impl WithConnection for Value {
    fn connection(&self) -> SharedConnection {
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
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::tools::dbtools::test_and_create_runtime_table;
    use anyhow::Result;
    use sqlx::{Connection, SqliteConnection};

    #[tokio::test]
    async fn test_insert_into_runtime_value_and_query() -> Result<()> {
        let conn = Arc::new(Mutex::new(
            SqliteConnection::connect("sqlite::memory:").await?,
        ));
        test_and_create_runtime_table(&conn).await?;
        let runtime_value = RuntimeValue::create(conn).await?;
        for i in 1..15 {
            let runtime_value_data = generate_test_runtime_value(i);
            runtime_value.append(runtime_value_data.clone()).await?;
            let fetch_runtime_value = runtime_value.query(i as i64).await?;
            println!("{:?}", fetch_runtime_value);
            assert!(compare_runtime_value(
                fetch_runtime_value.data(),
                &runtime_value_data
            ));
        }
        Ok(())
    }
    #[tokio::test]
    async fn test_update_runtime_value_and_query() -> Result<()> {
        let conn = Arc::new(Mutex::new(
            SqliteConnection::connect("sqlite::memory:").await?,
        ));
        test_and_create_runtime_table(&conn).await?;
        let runtime_value = RuntimeValue::create(conn).await?;
        for i in 1..15 {
            let runtime_value_data = generate_test_runtime_value(i);
            runtime_value.append(runtime_value_data.clone()).await?;
            let fetch_runtime_value = runtime_value.query(i as i64).await?;
            println!("Before: {:?}", fetch_runtime_value);
            assert!(compare_runtime_value(
                fetch_runtime_value.data(),
                &runtime_value_data
            ));

            let new_runtime_value = generate_test_runtime_value(i + 200);
            runtime_value
                .set(
                    fetch_runtime_value.data().id as i64,
                    new_runtime_value.clone(),
                )
                .await?;
            let fetch_runtime_value = runtime_value.query(i as i64).await?;

            println!("After: {:?}", fetch_runtime_value);
            assert!(compare_runtime_value(
                fetch_runtime_value.data(),
                &new_runtime_value
            ));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_remove_runtime_value_and_query() -> Result<()> {
        let conn = Arc::new(Mutex::new(
            SqliteConnection::connect("sqlite::memory:").await?,
        ));
        test_and_create_runtime_table(&conn).await?;
        let runtime_value = RuntimeValue::create(conn).await?;
        for i in 1..15 {
            let runtime_value_data = generate_test_runtime_value(i);
            runtime_value.append(runtime_value_data.clone()).await?;
            let fetch_runtime_value = runtime_value.query(i as i64).await?;
            println!("Before: {:?}", fetch_runtime_value);
            assert!(compare_runtime_value(
                fetch_runtime_value.data(),
                &runtime_value_data
            ));

            runtime_value
                .remove(fetch_runtime_value.data().id as i64)
                .await?;
            let fetch_runtime_value = runtime_value.query(i as i64).await;
            let error_expected = sqlx::Error::RowNotFound;

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
    #[tokio::test]
    async fn test_set_and_get_runtime_value() -> Result<()> {
        let conn = Arc::new(Mutex::new(
            SqliteConnection::connect("sqlite::memory:").await?,
        ));
        test_and_create_runtime_table(&conn).await?;
        let runtime_value = RuntimeValue::create(conn).await?;
        for i in 1..15 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            runtime_value.set_by_key(&key, value.clone()).await?;
            let fetch_runtime_value = runtime_value.get_by_key(&key).await.unwrap();
            println!("Before: {:?}", &fetch_runtime_value);
            assert_eq!(value, fetch_runtime_value);

            let value = format!("value{}", i);
            runtime_value.set_by_key(&key, value.clone()).await?;
            let fetch_runtime_value = runtime_value.get_by_key(&key).await.unwrap();

            println!("After: {:?}", &fetch_runtime_value);
            assert_eq!(value, fetch_runtime_value);
        }
        Ok(())
    }
}

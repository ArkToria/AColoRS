use std::sync::Arc;

use sqlx::SqliteConnection;
use tokio::sync::Mutex;

use crate::table_member::{grouplist::GroupList, runtime::RuntimeValue};

#[derive(Debug)]
pub struct Profile {
    pub group_list: GroupList,
    pub runtime_value: RuntimeValue,
}

impl Profile {
    pub async fn create(connection: SqliteConnection) -> sqlx::Result<Profile> {
        let connection = Arc::new(Mutex::new(connection));
        Ok(Profile {
            group_list: GroupList::create(connection.clone()).await?,
            runtime_value: RuntimeValue::create(connection).await?,
        })
    }
}

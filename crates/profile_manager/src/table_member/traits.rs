use std::sync::Arc;

use tokio::sync::Mutex;

use sqlx::Database;

type DatabaseDriver = sqlx::Sqlite;
type SharedConnection = Arc<Mutex<<DatabaseDriver as Database>::Connection>>;

pub trait AColoRSListModel<T, D>: HasTable
where
    T: Clone + AttachedToTable<D>,
    D: Clone,
{
    fn size(&self) -> sqlx::Result<usize>;
    fn append(&mut self, item: &D) -> sqlx::Result<usize>;
    fn set(&mut self, id: usize, item: &D) -> sqlx::Result<()>;
    fn remove(&mut self, id: usize) -> anyhow::Result<()>;
    fn query(&self, id: usize) -> anyhow::Result<T>;
}

pub trait WithConnection {
    fn connection(&self) -> SharedConnection;
}

pub trait AttachedToTable<D>: WithConnection
where
    D: Clone,
{
    fn attached_to_table_name() -> &'static str;
    fn field_names() -> &'static [&'static str];
    fn get_update_sql() -> &'static str;
    fn get_insert_sql() -> &'static str;
    fn get_remove_sql() -> &'static str;
    fn get_query_sql() -> &'static str;
}

pub trait HasTable: WithConnection {
    fn has_table_name() -> &'static str;
}

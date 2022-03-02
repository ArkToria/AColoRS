use std::sync::Arc;

use futures::TryStreamExt;
use sqlx::Database;
use sqlx::Row;
use tokio::sync::Mutex;

use crate::table_member::group::Group;
use crate::tools::dbtools::test_and_create_group_table;
use crate::tools::dbtools::test_and_create_node_table;
use core_data::data_type::group::*;

type DatabaseDriver = sqlx::Sqlite;
type SharedConnection = Arc<Mutex<<DatabaseDriver as Database>::Connection>>;
type TRow = <DatabaseDriver as Database>::Row;
#[derive(Debug)]
pub struct GroupList {
    connection: SharedConnection,
}

impl GroupList {
    pub async fn create(connection: SharedConnection) -> sqlx::Result<GroupList> {
        test_and_create_group_table(&connection).await?;
        test_and_create_node_table(&connection).await?;
        Ok(GroupList { connection })
    }
    pub async fn list_all_groups(&self) -> anyhow::Result<Vec<Group>> {
        let rows: Vec<TRow>;
        {
            let conn_mut = &mut *self.connection.lock().await;
            rows = sqlx::query("SELECT * FROM groups")
                .fetch_all(conn_mut)
                .await?;
        }
        let result = rows
            .into_iter()
            .map(|row| {
                let data = GroupData {
                    id: row.get(0),
                    name: row.get(1),
                    is_subscription: row.get(2),
                    group_type: row.get(3),
                    url: row.get(4),
                    cycle_time: row.get(5),
                    create_at: row.get(6),
                    modified_at: row.get(7),
                };
                Group::new(data, self.connection.clone())
            })
            .collect();
        Ok(result)
    }

    /// For node query
    pub fn default_group(&self) -> Group {
        let data = GroupData::default();
        Group::new(data, self.connection.clone())
    }

    pub async fn append(&self, mut item: GroupData) -> sqlx::Result<i64> {
        item.update_create_at();
        item.update_modified_at();

        let query = sqlx::query(GROUP_INSERT_SQL)
            .bind(item.name)
            .bind(item.is_subscription)
            .bind(item.group_type)
            .bind(item.url)
            .bind(item.cycle_time)
            .bind(item.create_at)
            .bind(item.modified_at);
        let conn_mut = &mut *self.connection.lock().await;

        let result = query.execute(conn_mut).await?.last_insert_rowid();
        Ok(result)
    }

    pub async fn set(&self, id: i64, mut item: GroupData) -> sqlx::Result<()> {
        item.update_modified_at();

        let query = sqlx::query(GROUP_UPDATE_SQL)
            .bind(item.name)
            .bind(item.is_subscription)
            .bind(item.group_type)
            .bind(item.url)
            .bind(item.cycle_time)
            .bind(item.create_at)
            .bind(item.modified_at)
            .bind(id);
        let conn_mut = &mut *self.connection.lock().await;

        query.execute(conn_mut).await?;
        Ok(())
    }

    pub async fn remove(&self, id: i64) -> sqlx::Result<()> {
        let group = self.query(id).await?;

        group.remove_all_nodes().await?;

        let query = sqlx::query(GROUP_REMOVE_SQL).bind(id);
        let conn_mut = &mut *self.connection.lock().await;

        query.execute(conn_mut).await?;
        Ok(())
    }

    pub async fn size(&self) -> sqlx::Result<i64> {
        let query = sqlx::query("SELECT COUNT(*) FROM groups");
        let conn_mut = &mut *self.connection.lock().await;
        let mut rows = query.fetch(conn_mut);
        let size = match rows.try_next().await? {
            Some(row) => row.try_get(0)?,
            None => return Err(sqlx::Error::RowNotFound),
        };
        Ok(size)
    }

    pub async fn query(&self, id: i64) -> sqlx::Result<Group> {
        let query = sqlx::query(GROUP_QUERY_SQL).bind(id);
        let conn_mut = &mut *self.connection.lock().await;
        let mut rows = query.fetch(conn_mut);
        let data = match rows.try_next().await? {
            Some(row) => GroupData {
                id: row.try_get(0)?,
                name: row.try_get(1)?,
                is_subscription: row.try_get(2)?,
                group_type: row.try_get(3)?,
                url: row.try_get(4)?,
                cycle_time: row.try_get(5)?,
                create_at: row.try_get(6)?,
                modified_at: row.try_get(7)?,
            },
            None => return Err(sqlx::Error::RowNotFound),
        };
        Ok(Group::new(data, self.connection.clone()))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        table_member::group::tests::{compare_group, generate_test_group},
        tools::dbtools::{test_and_create_group_table, test_and_create_node_table},
    };
    use anyhow::Result;
    use sqlx::{Connection, SqliteConnection};

    #[tokio::test]
    async fn test_insert_into_group_and_query() -> Result<()> {
        let conn = Arc::new(Mutex::new(
            SqliteConnection::connect("sqlite::memory:").await?,
        ));
        test_and_create_group_table(&conn).await?;
        test_and_create_node_table(&conn).await?;
        let group_list = GroupList::create(conn).await?;
        for i in 1..15 {
            let group_data = generate_test_group(i);
            group_list.append(group_data.clone()).await?;
            let fetch_group = group_list.query(i as i64).await?;
            println!("{:?}", fetch_group);
            assert!(compare_group(fetch_group.data(), &group_data));
        }
        Ok(())
    }
    #[tokio::test]
    async fn test_update_group_and_query() -> Result<()> {
        let conn = Arc::new(Mutex::new(
            SqliteConnection::connect("sqlite::memory:").await?,
        ));
        test_and_create_group_table(&conn).await?;
        test_and_create_node_table(&conn).await?;
        let group_list = GroupList::create(conn).await?;
        for i in 1..15 {
            let group_data = generate_test_group(i);
            group_list.append(group_data.clone()).await?;
            let fetch_group = group_list.query(i as i64).await?;
            println!("Before: {:?}", fetch_group);
            assert!(compare_group(fetch_group.data(), &group_data));

            let new_group = generate_test_group(i + 200);
            group_list
                .set(fetch_group.data().id as i64, new_group.clone())
                .await?;
            let fetch_group = group_list.query(i as i64).await?;

            println!("After: {:?}", fetch_group);
            assert!(compare_group(fetch_group.data(), &new_group));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_remove_group_and_query() -> Result<()> {
        let conn = Arc::new(Mutex::new(
            SqliteConnection::connect("sqlite::memory:").await?,
        ));
        test_and_create_group_table(&conn).await?;
        test_and_create_node_table(&conn).await?;
        let group_list = GroupList::create(conn).await?;
        for i in 1..15 {
            let group_data = generate_test_group(i);
            group_list.append(group_data.clone()).await?;
            let fetch_group = group_list.query(i as i64).await?;
            println!("Before: {:?}", fetch_group);
            assert!(compare_group(fetch_group.data(), &group_data));

            group_list.remove(fetch_group.data().id as i64).await?;
            let fetch_group = group_list.query(i as i64).await;
            let error_expected = sqlx::Error::RowNotFound;

            if let Err(e) = fetch_group {
                assert_eq!(error_expected.to_string(), e.to_string());
            } else {
                panic!("No Errors when group removed");
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_insert_into_group_and_list_all() -> Result<()> {
        let conn = Arc::new(Mutex::new(
            SqliteConnection::connect("sqlite::memory:").await?,
        ));
        test_and_create_group_table(&conn).await?;
        test_and_create_node_table(&conn).await?;
        let group_list = GroupList::create(conn).await?;
        let mut group_vec: Vec<Group> = Vec::new();
        for i in 1..15 {
            let group_data = generate_test_group(i);
            group_list.append(group_data.clone()).await?;
            let fetch_group = group_list.query(i as i64).await?;
            group_vec.push(fetch_group.clone());
            assert!(compare_group(fetch_group.data(), &group_data));
        }
        let nodes = group_list.list_all_groups().await?;
        for i in 0..14 {
            let q_group = group_vec[i].data();
            println!("{}: \n{:?}", i, q_group);
            let s_group = nodes[i].data();
            println!("{:?}\n", s_group);
            assert!(compare_group(group_vec[i].data(), nodes[i].data()));
        }
        Ok(())
    }
}

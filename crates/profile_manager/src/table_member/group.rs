use std::sync::Arc;

use core_data::data_type::{group::*, node::*};
use futures::TryStreamExt;
use sqlx::{Database, Executor, Row};
use tokio::sync::Mutex;

use super::node::Node;

type DatabaseDriver = sqlx::Sqlite;
type SharedConnection = Arc<Mutex<<DatabaseDriver as Database>::Connection>>;
type TRow = <DatabaseDriver as Database>::Row;
#[derive(Debug, Clone)]
pub struct Group {
    data: GroupData,
    connection: SharedConnection,
}

impl Group {
    pub fn new(data: GroupData, connection: SharedConnection) -> Group {
        Group { data, connection }
    }

    /// Get a reference to the group's data.
    pub fn data(&self) -> &GroupData {
        &self.data
    }

    pub fn to_data(self) -> GroupData {
        self.data
    }

    pub async fn remove_all_nodes(&self) -> sqlx::Result<()> {
        let query = sqlx::query("DELETE FROM nodes WHERE GroupID = ?").bind(self.data.id);
        let conn_mut = &mut *self.connection.lock().await;
        conn_mut.execute(query).await?;
        Ok(())
    }

    pub async fn list_all_nodes(&self) -> sqlx::Result<Vec<Node>> {
        let rows: Vec<TRow>;
        {
            let conn_mut = &mut *self.connection.lock().await;
            rows = sqlx::query("SELECT * FROM nodes WHERE GroupID = ?")
                .bind(self.data.id)
                .fetch_all(conn_mut)
                .await?;
        }
        let result = rows
            .into_iter()
            .map(|row| {
                let data = NodeData {
                    id: row.get(0),
                    name: row.get(1),
                    group_id: row.get(2),
                    group_name: row.get(3),
                    routing_id: row.get(4),
                    routing_name: row.get(5),
                    protocol: row.get(6),
                    address: row.get(7),
                    port: row.get(8),
                    password: row.get(9),
                    raw: row.get(10),
                    url: row.get(11),
                    latency: row.get(12),
                    upload: row.get(13),
                    download: row.get(14),
                    create_at: row.get(15),
                    modified_at: row.get(16),
                };
                Node::new(data, self.connection.clone())
            })
            .collect();
        Ok(result)
    }
    pub async fn append(&self, mut item: NodeData) -> sqlx::Result<i64> {
        item.group_id = self.data().id;
        item.group_name = self.data().name.clone();

        // TODO: routing
        item.routing_name = "default_routings".to_string();
        item.routing_id = 0;

        item.initialize();

        let query = sqlx::query(NODE_INSERT_SQL)
            .bind(item.name)
            .bind(item.group_id)
            .bind(item.group_name)
            .bind(item.routing_id)
            .bind(item.routing_name)
            .bind(item.protocol)
            .bind(item.address)
            .bind(item.port)
            .bind(item.password)
            .bind(item.raw)
            .bind(item.url)
            .bind(item.latency)
            .bind(item.upload)
            .bind(item.download)
            .bind(item.create_at)
            .bind(item.modified_at);
        let conn_mut = &mut *self.connection.lock().await;

        let result = query.execute(conn_mut).await?.last_insert_rowid();
        Ok(result)
    }
    pub async fn set(&self, id: i64, mut item: NodeData) -> sqlx::Result<()> {
        item.group_id = self.data().id;
        item.update_modified_at();

        let query = sqlx::query(NODE_UPDATE_SQL)
            .bind(item.name)
            .bind(item.group_id)
            .bind(item.group_name)
            .bind(item.routing_id)
            .bind(item.routing_name)
            .bind(item.protocol)
            .bind(item.address)
            .bind(item.port)
            .bind(item.password)
            .bind(item.raw)
            .bind(item.url)
            .bind(item.latency)
            .bind(item.upload)
            .bind(item.download)
            .bind(item.create_at)
            .bind(item.modified_at)
            .bind(id);
        let conn_mut = &mut *self.connection.lock().await;

        query.execute(conn_mut).await?;
        Ok(())
    }
    pub async fn size(&self) -> sqlx::Result<i64> {
        let query = sqlx::query("SELECT COUNT(*) FROM nodes WHERE GroupID = ?").bind(self.data.id);
        let conn_mut = &mut *self.connection.lock().await;
        let mut rows = query.fetch(conn_mut);
        let size;
        match rows.try_next().await? {
            Some(row) => size = row.try_get(0)?,
            None => return Err(sqlx::Error::RowNotFound),
        }
        Ok(size)
    }

    pub async fn remove(&self, id: i64) -> sqlx::Result<()> {
        let query = sqlx::query(NODE_REMOVE_SQL).bind(id);
        let conn_mut = &mut *self.connection.lock().await;

        query.execute(conn_mut).await?;
        Ok(())
    }

    pub async fn query(&self, id: i64) -> sqlx::Result<Node> {
        let query = sqlx::query(NODE_QUERY_SQL).bind(id);
        let conn_mut = &mut *self.connection.lock().await;
        let mut rows = query.fetch(conn_mut);
        let data = match rows.try_next().await? {
            Some(row) => NodeData {
                id: row.try_get(0)?,
                name: row.try_get(1)?,
                group_id: row.try_get(2)?,
                group_name: row.try_get(3)?,
                routing_id: row.try_get(4)?,
                routing_name: row.try_get(5)?,
                protocol: row.try_get(6)?,
                address: row.try_get(7)?,
                port: row.try_get(8)?,
                password: row.try_get(9)?,
                raw: row.try_get(10)?,
                url: row.try_get(11)?,
                latency: row.try_get(12)?,
                upload: row.try_get(13)?,
                download: row.try_get(14)?,
                create_at: row.try_get(15)?,
                modified_at: row.try_get(16)?,
            },
            None => return Err(sqlx::Error::RowNotFound),
        };
        Ok(Node::new(data, self.connection.clone()))
    }
}
#[cfg(test)]
pub mod tests {
    use crate::{
        table_member::{
            grouplist::GroupList,
            node::tests::{compare_node, generate_test_node},
        },
        tools::dbtools::{test_and_create_group_table, test_and_create_node_table},
    };

    use super::*;
    use anyhow::Result;
    use sqlx::{Connection, SqliteConnection};
    #[tokio::test]
    async fn test_insert_into_node_and_query() -> Result<()> {
        let conn = Arc::new(Mutex::new(
            SqliteConnection::connect("sqlite::memory:").await?,
        ));
        test_and_create_group_table(&conn).await?;
        test_and_create_node_table(&conn).await?;
        let group_list = GroupList::create(conn).await?;
        group_list.append(generate_test_group(1)).await?;
        group_list.append(generate_test_group(2)).await?;
        group_list.append(generate_test_group(3)).await?;
        let group = group_list.query(2).await?;
        for i in 1..15 {
            let node_data = generate_test_node(i);
            group.append(node_data.clone()).await?;
            let fetch_node = group.query(i as i64).await?;
            println!("{:?}", fetch_node);
            assert!(compare_node(fetch_node.data(), &node_data));
        }
        Ok(())
    }
    #[tokio::test]
    async fn test_update_node_and_query() -> Result<()> {
        let conn = Arc::new(Mutex::new(
            SqliteConnection::connect("sqlite::memory:").await?,
        ));
        test_and_create_group_table(&conn).await?;
        test_and_create_node_table(&conn).await?;
        let group_list = GroupList::create(conn).await?;
        group_list.append(generate_test_group(1)).await?;
        group_list.append(generate_test_group(2)).await?;
        group_list.append(generate_test_group(3)).await?;
        let group = group_list.query(2).await?;
        for i in 1..15 {
            let node_data = generate_test_node(i);
            group.append(node_data.clone()).await?;
            let fetch_node = group.query(i as i64).await?;
            assert!(compare_node(fetch_node.data(), &node_data));
            println!("Before: {:?}", fetch_node);

            let new_node = generate_test_node(i + 200);
            group
                .set(fetch_node.data().id as i64, new_node.clone())
                .await?;
            let fetch_node = group.query(i as i64).await?;

            println!("After: {:?}", fetch_node);
            assert!(compare_node(fetch_node.data(), &new_node));
        }
        Ok(())
    }
    #[tokio::test]
    async fn test_remove_node_and_query() -> Result<()> {
        let conn = Arc::new(Mutex::new(
            SqliteConnection::connect("sqlite::memory:").await?,
        ));
        test_and_create_group_table(&conn).await?;
        test_and_create_node_table(&conn).await?;
        let group_list = GroupList::create(conn).await?;
        group_list.append(generate_test_group(1)).await?;
        group_list.append(generate_test_group(2)).await?;
        group_list.append(generate_test_group(3)).await?;
        let group = group_list.query(2).await?;
        for i in 1..15 {
            let node_data = generate_test_node(i);
            group.append(node_data.clone()).await?;
            let fetch_node = group.query(i as i64).await?;
            assert!(compare_node(fetch_node.data(), &node_data));
            println!("Before: {:?}", fetch_node);

            group.remove(fetch_node.data().id as i64).await?;
            let fetch_node = group.query(i as i64).await;
            let error_expected = sqlx::Error::RowNotFound;

            if let Err(e) = fetch_node {
                assert_eq!(error_expected.to_string(), e.to_string());
            } else {
                panic!("No Errors when group removed");
            }
        }
        Ok(())
    }
    #[tokio::test]
    async fn test_insert_into_node_and_list_all() -> Result<()> {
        let conn = Arc::new(Mutex::new(
            SqliteConnection::connect("sqlite::memory:").await?,
        ));
        test_and_create_group_table(&conn).await?;
        test_and_create_node_table(&conn).await?;
        let group_list = GroupList::create(conn).await?;
        group_list.append(generate_test_group(1)).await?;
        group_list.append(generate_test_group(2)).await?;
        group_list.append(generate_test_group(3)).await?;
        let group = group_list.query(2).await?;
        let mut node_list: Vec<Node> = Vec::new();
        for i in 1..15 {
            let node_data = generate_test_node(i);
            group.append(node_data.clone()).await?;
            let fetch_node = group.query(i as i64).await?;
            node_list.push(fetch_node.clone());
            assert!(compare_node(fetch_node.data(), &node_data));
        }
        let nodes = group.list_all_nodes().await?;
        for i in 0..14 {
            let q_node = node_list[i].data();
            println!("{}: \n{:?}", i, q_node);
            let s_node = nodes[i].data();
            println!("{:?}\n", s_node);
            assert!(compare_node(node_list[i].data(), nodes[i].data()));
        }
        Ok(())
    }
    pub fn compare_group(a: &GroupData, b: &GroupData) -> bool {
        let mut ac = a.clone();
        let mut bc = b.clone();
        ac.id = 0;
        bc.id = 0;
        ac == bc
    }
    pub fn generate_test_group(number: u16) -> GroupData {
        let test_string = format!("test{}", number);
        let mut result = GroupData {
            id: number as i32,
            name: format!("{} group", &test_string),
            is_subscription: false,
            group_type: 0,
            url: format!("https://localhost:{}", number),
            cycle_time: number as i32,
            create_at: 0,
            modified_at: 0,
        };
        result.update_create_at();
        result.update_modified_at();
        result
    }
}

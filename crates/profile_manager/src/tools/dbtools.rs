use std::{process, rc::Rc};

use anyhow::Result;
use rusqlite::Connection;
use spdlog::error;

use crate::data_type::traits::AttachedToTable;

use super::schema::{GROUP_SCHEMA, NODE_SCHEMA, RUNTIME_SCHEMA};

pub fn test_and_create_node_table(conn: &Connection) -> Result<()> {
    conn.execute(NODE_SCHEMA, [])?;
    Ok(())
}

pub fn test_and_create_group_table(conn: &Connection) -> Result<()> {
    conn.execute(GROUP_SCHEMA, [])?;
    Ok(())
}

pub fn test_and_create_runtime_table(conn: &Connection) -> Result<()> {
    conn.execute(RUNTIME_SCHEMA, [])?;
    Ok(())
}

pub fn count_table(connection: &Connection, name: &str) -> Result<usize> {
    let sql = format!("SELECT COUNT(*) FROM {}", name);
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
pub fn insert_into_table<T, D>(connection: &Connection, item: &D) -> Result<()>
where
    T: AttachedToTable<D>,
    D: Clone,
{
    let sql = get_insert_sql::<T, D>();
    let mut statement = connection.prepare(&sql)?;
    T::execute_statement(item, &mut statement)?;
    Ok(())
}
fn get_insert_sql<T, D>() -> String
where
    T: AttachedToTable<D>,
    D: Clone,
{
    let field_names = T::field_names();
    format!(
        "INSERT INTO {}({}) VALUES({})",
        T::attached_to_table_name(),
        format_with_comma(field_names),
        generate_question_marks_with_comma(field_names.len())
    )
}
pub fn update_table<T, D>(connection: &Connection, id: usize, item: &D) -> Result<()>
where
    T: AttachedToTable<D>,
    D: Clone,
{
    let sql = get_update_sql::<T, D>();
    let mut statement = connection.prepare(&sql)?;
    T::execute_statement_with_id(item, id, &mut statement)?;
    Ok(())
}
fn get_update_sql<T, D>() -> String
where
    T: AttachedToTable<D>,
    D: Clone,
{
    let field_names = T::field_names();
    format!(
        "UPDATE {} SET {} WHERE ID = ?;",
        T::attached_to_table_name(),
        format_name_question_mark_pair_with_comma(field_names)
    )
}
pub fn remove_from_table<T, D>(connection: &Connection, id: usize) -> Result<()>
where
    T: AttachedToTable<D>,
    D: Clone,
{
    let sql = format!("DELETE FROM {} WHERE ID = ?", T::attached_to_table_name());
    let mut statement = connection.prepare(&sql)?;
    statement.execute(&[&id])?;
    Ok(())
}
pub fn query_from_table<T, D>(connection: Rc<Connection>, id: usize) -> Result<T>
where
    T: AttachedToTable<D>,
    D: Clone,
{
    let field_names = T::field_names();
    let sql = format!(
        "SELECT ID,{} FROM {} WHERE ID = ?",
        format_with_comma(field_names),
        T::attached_to_table_name()
    );
    let mut statement = connection.prepare(&sql)?;
    let item_data = T::query_map(connection.clone(), &mut statement, id)?;
    Ok(item_data)
}
fn format_name_question_mark_pair_with_comma(strings: &[&str]) -> String {
    let mut result = String::new();
    let mut comma_flag = false;
    IntoIterator::into_iter(strings).for_each(|st| {
        if comma_flag {
            result += ",";
        }
        result += &format!("{} = ?", st);
        comma_flag = true;
    });
    result
}
fn format_with_comma(strings: &[&str]) -> String {
    let mut result = String::new();
    let mut comma_flag = false;
    IntoIterator::into_iter(strings).for_each(|st| {
        if comma_flag {
            result += ",";
        }
        result += st;
        comma_flag = true;
    });
    result
}
fn generate_question_marks_with_comma(count: usize) -> String {
    let mut result = String::new();
    let mut comma_flag = false;
    (0..count).for_each(|_| {
        if comma_flag {
            result += ",";
        }
        result += "?";
        comma_flag = true;
    });
    result
}
#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use anyhow::Result;
    use rusqlite::{params, Connection};

    use crate::{
        data_type::{
            group::GroupData, grouplist::GroupList, node::NodeData, traits::AColoRSListModel,
        },
        tools::dbtools::{count_table, test_and_create_group_table, test_and_create_node_table},
    };
    #[test]
    fn test_sth() -> Result<()> {
        Ok(())
    }
    #[test]
    fn test_count_table() -> Result<()> {
        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE testtable (
                  id              INTEGER PRIMARY KEY,
                  name            TEXT NOT NULL
                  )",
            [],
        )?;
        assert_eq!(0, count_table(&conn, "testtable")?);
        for i in 1..15 {
            println!("{}!", i);
            let name = format!("test name {}", i);
            conn.execute("INSERT INTO testtable (name) VALUES (?1)", params![name])?;
            assert_eq!(i, count_table(&conn, "testtable")?);
        }
        Ok(())
    }
    #[test]
    fn test_insert_into_group_and_query() -> Result<()> {
        let conn = Rc::new(Connection::open_in_memory()?);
        test_and_create_group_table(&conn)?;
        test_and_create_node_table(&conn)?;
        let mut group_list = GroupList::new(conn);
        for i in 1..15 {
            let group_data = generate_test_group(i);
            group_list.append(&group_data)?;
            let fetch_group = group_list.query(i as usize)?;
            println!("{:?}", fetch_group);
            assert!(compare_group(fetch_group.data(), &group_data));
        }
        Ok(())
    }
    fn compare_group(a: &GroupData, b: &GroupData) -> bool {
        let mut ac = a.clone();
        let mut bc = b.clone();
        ac.id = 0;
        bc.id = 0;
        ac == bc
    }
    fn generate_test_group(number: u16) -> GroupData {
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
    #[test]
    fn test_insert_into_node_and_query() -> Result<()> {
        let conn = Rc::new(Connection::open_in_memory()?);
        test_and_create_group_table(&conn)?;
        test_and_create_node_table(&conn)?;
        let mut group_list = GroupList::new(conn);
        group_list.append(&generate_test_group(1))?;
        group_list.append(&generate_test_group(2))?;
        group_list.append(&generate_test_group(3))?;
        let mut group = group_list.query(2)?;
        for i in 1..15 {
            let node_data = generate_test_node(i);
            group.append(&node_data)?;
            let fetch_node = group.query(i as usize)?;
            println!("{:?}", fetch_node);
            assert!(compare_node(fetch_node.data(), &node_data));
        }
        Ok(())
    }
    fn compare_node(a: &NodeData, b: &NodeData) -> bool {
        let mut ac = a.clone();
        let mut bc = b.clone();
        ac.id = 0;
        bc.id = 0;
        ac == bc
    }
    fn generate_test_node(number: u16) -> NodeData {
        let test_string = format!("test{}", number);
        let test_address = format!("localhost:{}", number);
        let mut result = NodeData {
            id: number as i32,
            name: format!("{} node", test_string),
            group_id: number as i32,
            group_name: format!("{} group", test_string),
            routing_id: number as i32,
            routing_name: format!("{} routing", test_string),
            protocol: number as i32,
            address: test_address.clone(),
            port: number,
            password: test_string.clone(),
            raw: test_string.clone(),
            url: format!("https://{}", test_address),
            latency: 100 * number as i32,
            upload: 200 * (number as i64),
            download: 300 * (number as i64),
            create_at: 0,
            modified_at: 0,
        };
        result.update_create_at();
        result.update_modified_at();
        result
    }
}

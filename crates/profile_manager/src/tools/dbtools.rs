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
        result += &format!("{} = ?", st);
        if comma_flag {
            result += ",";
        }
        comma_flag = true;
    });
    result
}
fn format_with_comma(strings: &[&str]) -> String {
    let mut result = String::new();
    let mut comma_flag = false;
    IntoIterator::into_iter(strings).for_each(|st| {
        result += st;
        if comma_flag {
            result += ",";
        }
        comma_flag = true;
    });
    result
}
fn generate_question_marks_with_comma(count: usize) -> String {
    let mut result = String::new();
    let mut comma_flag = false;
    (0..count).for_each(|_| {
        result += "?";
        if comma_flag {
            result += ",";
        }
        comma_flag = true;
    });
    result
}
#[cfg(test)]
mod tests {
    use anyhow::Result;
    use rusqlite::{params, Connection};

    use crate::tools::dbtools::count_table;
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
}

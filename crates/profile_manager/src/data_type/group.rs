use std::rc::Rc;

use anyhow::anyhow;
use rusqlite::{params, Connection};
use utils::time::get_current_time;

use super::{
    node::{Node, NodeData},
    traits::{AColoRSListModel, AttachedToTable, HasTable, WithConnection},
};

#[derive(Debug, Clone)]
pub struct Group {
    data: GroupData,
    connection: Rc<Connection>,
}

impl Group {
    /// Get a reference to the group's data.
    pub fn data(&self) -> &GroupData {
        &self.data
    }
}
#[derive(Debug, Clone)]
pub struct GroupData {
    pub id: i32,
    pub name: String,
    pub is_subscription: bool,
    pub group_type: i32,
    pub url: String,
    pub cycle_time: i32,
    pub create_at: i64,
    pub modified_at: i64,
}
const GROUP_TABLE_NAME: &str = "groups";
const GROUP_FIELD_NAMES: &[&str] = &[
    "Name",
    "IsSubscription",
    "Type",
    "Url",
    "CycleTime",
    "CreatedAt",
    "ModifiedAt",
];

impl GroupData {
    pub fn update_modified_at(&mut self) {
        self.modified_at = get_current_time() as i64;
    }

    pub fn update_create_at(&mut self) {
        self.create_at = get_current_time() as i64;
    }
}

const NODE_TABLE_NAME: &str = "nodes";

impl AttachedToTable<GroupData> for Group {
    fn attached_to_table_name() -> &'static str {
        GROUP_TABLE_NAME
    }
    fn field_names() -> &'static [&'static str] {
        GROUP_FIELD_NAMES
    }

    fn execute_statement(
        item_data: &GroupData,
        statement: &mut rusqlite::Statement,
    ) -> rusqlite::Result<usize> {
        statement.execute(params![
            item_data.name,
            item_data.is_subscription,
            item_data.group_type,
            item_data.url,
            item_data.cycle_time,
            item_data.create_at,
            item_data.modified_at,
        ])
    }
    fn execute_statement_with_id(
        item_data: &GroupData,
        id: usize,
        statement: &mut rusqlite::Statement,
    ) -> rusqlite::Result<usize> {
        statement.execute(params![
            item_data.name,
            item_data.is_subscription,
            item_data.group_type,
            item_data.url,
            item_data.cycle_time,
            item_data.create_at,
            item_data.modified_at,
            id,
        ])
    }

    fn query_map(
        connection: Rc<Connection>,
        statement: &mut rusqlite::Statement,
        id: usize,
    ) -> anyhow::Result<Group> {
        let iter = statement.query_map(&[&id], |row| {
            Ok(GroupData {
                id: row.get(0)?,
                name: row.get(1)?,
                is_subscription: row.get(2)?,
                group_type: row.get(3)?,
                url: row.get(4)?,
                cycle_time: row.get(5)?,
                create_at: row.get(6)?,
                modified_at: row.get(7)?,
            })
        })?;
        for data in iter {
            match data {
                Ok(d) => {
                    return Ok(Group {
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
impl HasTable for Group {
    fn has_table_name() -> &'static str {
        NODE_TABLE_NAME
    }
}

impl WithConnection for Group {
    fn connection(&self) -> Rc<Connection> {
        self.connection.clone()
    }
}

impl AColoRSListModel<Node, NodeData> for Group {}

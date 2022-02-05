use utils::time::get_current_time;

#[derive(Debug, Clone, Default, PartialEq)]
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
pub const GROUP_TABLE_NAME: &str = "groups";
pub const GROUP_FIELD_NAMES: &[&str] = &[
    "Name",
    "IsSubscription",
    "Type",
    "Url",
    "CycleTime",
    "CreatedAt",
    "ModifiedAt",
];
pub const GROUP_UPDATE_SQL: &str = "UPDATE groups SET Name = ?,IsSubscription = ?,Type = ?,Url = ?,CycleTime = ?,CreatedAt = ?,ModifiedAt = ? WHERE ID = ?;";
pub const GROUP_INSERT_SQL: &str = "INSERT INTO groups(Name,IsSubscription,Type,Url,CycleTime,CreatedAt,ModifiedAt) VALUES(?,?,?,?,?,?,?)";
pub const GROUP_REMOVE_SQL: &str = "DELETE FROM groups WHERE ID = ?";
pub const GROUP_QUERY_SQL: &str = "SELECT ID,Name,IsSubscription,Type,Url,CycleTime,CreatedAt,ModifiedAt FROM groups WHERE ID = ?";

impl GroupData {
    pub fn update_modified_at(&mut self) {
        self.modified_at = get_current_time() as i64;
    }

    pub fn update_create_at(&mut self) {
        self.create_at = get_current_time() as i64;
    }
}

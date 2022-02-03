#[derive(Debug, Clone, PartialEq)]
pub struct ValueData {
    pub id: i32,
    pub name: String,
    pub value_type: i32,
    pub value: String,
}
pub const RUNTIME_TABLE_NAME: &str = "runtime";
pub const RUNTIME_FIELD_NAMES: &[&str] = &["Name", "Type", "Value"];
pub const RUNTIME_UPDATE_SQL: &str =
    "UPDATE runtime SET Name = ?, Type = ?, Value = ? WHERE ID = ?;";
pub const RUNTIME_INSERT_SQL: &str = "INSERT INTO runtime(Name,Type,Value) VALUES(?,?,?)";
pub const RUNTIME_REMOVE_SQL: &str = "DELETE FROM runtime WHERE ID = ?";
pub const RUNTIME_QUERY_SQL: &str = "SELECT ID,Name,Type,Value FROM runtime WHERE ID = ?";

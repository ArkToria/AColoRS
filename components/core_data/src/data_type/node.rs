use utils::time::get_current_time;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NodeData {
    pub id: i32,
    pub name: String,
    pub group_id: i32,
    pub group_name: String,
    pub routing_id: i32,
    pub routing_name: String,
    pub protocol: i32,
    pub address: String,
    pub port: i32,
    pub password: String,
    pub raw: String,
    pub url: String,
    pub latency: i32,
    pub upload: i64,
    pub download: i64,
    pub create_at: i64,
    pub modified_at: i64,
}
pub const NODE_TABLE_NAME: &str = "nodes";
pub const NODE_FIELD_NAMES: &[&str] = &[
    "Name",
    "GroupID",
    "GroupName",
    "RoutingID",
    "RoutingName",
    "Protocol",
    "Address",
    "Port",
    "Password",
    "Raw",
    "URL",
    "Latency",
    "Upload",
    "Download",
    "CreatedAt",
    "ModifiedAt",
];
pub const NODE_UPDATE_SQL: &str = "UPDATE nodes SET Name = ?,GroupID = ?,GroupName = ?,RoutingID = ?,RoutingName = ?,Protocol = ?,Address = ?,Port = ?,Password = ?,Raw = ?,URL = ?,Latency = ?,Upload = ?,Download = ?,CreatedAt = ?,ModifiedAt = ? WHERE ID = ?;";
pub const NODE_INSERT_SQL: &str = "INSERT INTO nodes(Name,GroupID,GroupName,RoutingID,RoutingName,Protocol,Address,Port,Password,Raw,URL,Latency,Upload,Download,CreatedAt,ModifiedAt) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)";
pub const NODE_REMOVE_SQL: &str = "DELETE FROM nodes WHERE ID = ?";
pub const NODE_QUERY_SQL: &str = "SELECT ID,Name,GroupID,GroupName,RoutingID,RoutingName,Protocol,Address,Port,Password,Raw,URL,Latency,Upload,Download,CreatedAt,ModifiedAt FROM nodes WHERE ID = ?";

impl NodeData {
    pub fn update_modified_at(&mut self) {
        self.modified_at = get_current_time() as i64;
    }

    pub fn update_create_at(&mut self) {
        self.create_at = get_current_time() as i64;
    }
    pub fn initialize(&mut self) {
        self.update_create_at();
        self.update_modified_at();
        self.latency = -1;
    }
}

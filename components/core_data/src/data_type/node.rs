pub use core_protobuf::acolors_proto::NodeData;
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

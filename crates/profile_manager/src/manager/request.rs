use core_data::{GroupData, NodeData};

#[derive(Debug)]
pub enum ProfileRequest {
    CountGroups,
    ListAllGroups,
    CountNodes(i32),
    ListAllNodes(i32),
    GetGroupById(i32),
    GetNodeById(i32),
    RemoveGroupById(i32),
    RemoveNodeById(i32),
    SetGroupById(i32, GroupData),
    SetNodeById(i32, NodeData),
    AppendGroup(GroupData),
    AppendNode(i32, NodeData),
    UpdateGroup(i32, Vec<NodeData>),
}

use core_data::{GroupData, NodeData};

#[derive(Debug)]
pub enum ProfileReply {
    Error(String),
    CountGroups(usize),
    ListAllGroups(Vec<GroupData>),
    CountNodes(usize),
    ListAllNodes(Vec<NodeData>),
    GetGroupById(GroupData),
    GetNodeById(NodeData),
    SetGroupById,
    SetNodeById,
    RemoveGroupById,
    RemoveNodeById,
    AppendGroup,
    AppendNode,
    UpdateGroup,
    GetRuntimeValue(String),
    SetRuntimeValue,
}

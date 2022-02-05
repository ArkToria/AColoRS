use crate::{GroupData, NodeData};

#[derive(Debug)]
pub enum ProfileReply {
    Error(String),
    CountGroups(usize),
    ListAllGroups(Vec<GroupData>),
    CountNodes(usize),
    ListAllNodes(Vec<NodeData>),
    GetGroupById(GroupData),
}

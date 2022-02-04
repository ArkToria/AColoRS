use crate::GroupData;

#[derive(Debug)]
pub enum ProfileReply {
    Error(String),
    CountGroups(usize),
    ListAllGroups(Vec<GroupData>),
}

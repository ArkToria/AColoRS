#[derive(Debug)]
pub enum ProfileRequest {
    CountGroups,
    ListAllGroups,
    CountNodes(i32),
    ListAllNodes(i32),
    GetGroupById(i32),
    GetNodeById(i32),
}

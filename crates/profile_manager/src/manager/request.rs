#[derive(Debug)]
pub enum ProfileRequest {
    CountGroups,
    ListAllGroups,
    CountNodes(i32),
    ListAllNodes(i32),
}

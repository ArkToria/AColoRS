#[derive(Debug)]
pub enum ProfileReply {
    Error(String),
    CountGroups(usize),
}

#[derive(Debug)]
pub enum ProfileReply {
    Error(ReplyError),
    CountGroups(usize),
}

#[derive(Debug)]
pub enum ReplyError {
    ChannelOpenFailed,
    NoImplementation,
}

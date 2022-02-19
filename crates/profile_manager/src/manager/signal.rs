use core_protobuf::acolors_proto;
use core_protobuf::acolors_proto::profile_signal::*;
#[derive(Debug, Clone)]
pub enum ProfileSignal {
    Empty,
    AppendGroup,
    RemoveGroupById(i32),
    RemoveNodeById(i32),
    SetGroupById(i32),
    SetNodeById(i32),
    AppendNode(i32),
    UpdateGroup(i32),
}

impl From<crate::ProfileSignal> for acolors_proto::ProfileSignal {
    fn from(profile_signal: crate::ProfileSignal) -> Self {
        match profile_signal {
            ProfileSignal::Empty => Self {
                signal: Some(Signal::Empty(Empty {})),
            },
            ProfileSignal::AppendGroup => Self {
                signal: Some(Signal::AppendGroup(AppendGroup {})),
            },
            ProfileSignal::RemoveGroupById(group_id) => Self {
                signal: Some(Signal::RemoveGroupById(RemoveGroupById { group_id })),
            },
            ProfileSignal::RemoveNodeById(node_id) => Self {
                signal: Some(Signal::RemoveNodeById(RemoveNodeById { node_id })),
            },
            ProfileSignal::SetGroupById(group_id) => Self {
                signal: Some(Signal::SetGroupById(SetGroupById { group_id })),
            },
            ProfileSignal::SetNodeById(node_id) => Self {
                signal: Some(Signal::SetNodeById(SetNodeById { node_id })),
            },
            ProfileSignal::AppendNode(group_id) => Self {
                signal: Some(Signal::AppendNode(AppendNode { group_id })),
            },
            ProfileSignal::UpdateGroup(group_id) => Self {
                signal: Some(Signal::UpdateGroup(UpdateGroup { group_id })),
            },
        }
    }
}
impl From<core_protobuf::acolors_proto::ProfileSignal> for crate::ProfileSignal {
    fn from(profile_signal: core_protobuf::acolors_proto::ProfileSignal) -> Self {
        match profile_signal.signal {
            Some(signal) => match signal {
                Signal::Empty(_) => Self::Empty,
                Signal::AppendGroup(_) => Self::AppendGroup,
                Signal::RemoveGroupById(m) => Self::RemoveGroupById(m.group_id),
                Signal::RemoveNodeById(m) => Self::RemoveNodeById(m.node_id),
                Signal::SetGroupById(m) => Self::SetGroupById(m.group_id),
                Signal::SetNodeById(m) => Self::SetNodeById(m.node_id),
                Signal::AppendNode(m) => Self::AppendNode(m.group_id),
                Signal::UpdateGroup(m) => Self::UpdateGroup(m.group_id),
            },
            None => Self::Empty,
        }
    }
}

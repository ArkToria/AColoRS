use core_protobuf::acolors_proto;
use core_protobuf::acolors_proto::a_color_signal::*;
#[derive(Debug, Clone)]
pub enum AColorSignal {
    Empty,
    AppendGroup,
    UpdateCoreStatus,
    UpdateInbounds,
    CoreConfigChanged,
    RemoveGroupById(i32),
    RemoveNodeById(i32),
    SetGroupById(i32),
    SetNodeById(i32),
    AppendNode(i32),
    UpdateGroup(i32),
}

impl From<crate::AColorSignal> for acolors_proto::AColorSignal {
    fn from(profile_signal: crate::AColorSignal) -> Self {
        match profile_signal {
            AColorSignal::Empty => Self {
                signal: Some(Signal::Empty(Empty {})),
            },
            AColorSignal::AppendGroup => Self {
                signal: Some(Signal::AppendGroup(AppendGroup {})),
            },
            AColorSignal::UpdateCoreStatus => Self {
                signal: Some(Signal::UpdateCoreStatus(UpdateCoreStatus {})),
            },
            AColorSignal::UpdateInbounds => Self {
                signal: Some(Signal::UpdateInbounds(UpdateInbounds {})),
            },
            AColorSignal::CoreConfigChanged => Self {
                signal: Some(Signal::CoreConfigChanged(CoreConfigChanged {})),
            },
            AColorSignal::RemoveGroupById(group_id) => Self {
                signal: Some(Signal::RemoveGroupById(RemoveGroupById { group_id })),
            },
            AColorSignal::RemoveNodeById(node_id) => Self {
                signal: Some(Signal::RemoveNodeById(RemoveNodeById { node_id })),
            },
            AColorSignal::SetGroupById(group_id) => Self {
                signal: Some(Signal::SetGroupById(SetGroupById { group_id })),
            },
            AColorSignal::SetNodeById(node_id) => Self {
                signal: Some(Signal::SetNodeById(SetNodeById { node_id })),
            },
            AColorSignal::AppendNode(group_id) => Self {
                signal: Some(Signal::AppendNode(AppendNode { group_id })),
            },
            AColorSignal::UpdateGroup(group_id) => Self {
                signal: Some(Signal::UpdateGroup(UpdateGroup { group_id })),
            },
        }
    }
}
impl From<core_protobuf::acolors_proto::AColorSignal> for crate::AColorSignal {
    fn from(profile_signal: core_protobuf::acolors_proto::AColorSignal) -> Self {
        profile_signal
            .signal
            .map(|s| match s {
                Signal::Empty(_) => Self::Empty,
                Signal::AppendGroup(_) => Self::AppendGroup,
                Signal::UpdateCoreStatus(_) => Self::UpdateCoreStatus,
                Signal::UpdateInbounds(_) => Self::UpdateInbounds,
                Signal::CoreConfigChanged(_) => Self::CoreConfigChanged,
                Signal::RemoveGroupById(m) => Self::RemoveGroupById(m.group_id),
                Signal::RemoveNodeById(m) => Self::RemoveNodeById(m.node_id),
                Signal::SetGroupById(m) => Self::SetGroupById(m.group_id),
                Signal::SetNodeById(m) => Self::SetNodeById(m.node_id),
                Signal::AppendNode(m) => Self::AppendNode(m.group_id),
                Signal::UpdateGroup(m) => Self::UpdateGroup(m.group_id),
            })
            .unwrap_or(Self::Empty)
    }
}

use core_protobuf::acolors_proto;
use core_protobuf::acolors_proto::a_color_signal::*;
#[derive(Debug, Clone)]
pub enum AColorSignal {
    Empty,
    AppendGroup { group_id: i64 },
    UpdateCoreStatus,
    UpdateInbounds,
    CoreConfigChanged,
    CoreChanged,
    RemoveGroupById { group_id: i64 },
    RemoveNodeById { node_id: i64 },
    SetGroupById { group_id: i64 },
    SetNodeById { node_id: i64 },
    AppendNode { group_id: i64, node_id: i64 },
    UpdateGroup { group_id: i64 },
    RuntimeValueChanged { key: String },
    EmptyGroup { group_id: i64 },
    Shutdown,
    SetApiStatus,
}

impl From<crate::AColorSignal> for acolors_proto::AColorSignal {
    fn from(profile_signal: crate::AColorSignal) -> Self {
        match profile_signal {
            AColorSignal::Empty => Self {
                signal: Some(Signal::Empty(Empty {})),
            },
            AColorSignal::AppendGroup { group_id } => Self {
                signal: Some(Signal::AppendGroup(AppendGroup { group_id })),
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
            AColorSignal::RemoveGroupById { group_id } => Self {
                signal: Some(Signal::RemoveGroupById(RemoveGroupById { group_id })),
            },
            AColorSignal::RemoveNodeById { node_id } => Self {
                signal: Some(Signal::RemoveNodeById(RemoveNodeById { node_id })),
            },
            AColorSignal::SetGroupById { group_id } => Self {
                signal: Some(Signal::SetGroupById(SetGroupById { group_id })),
            },
            AColorSignal::SetNodeById { node_id } => Self {
                signal: Some(Signal::SetNodeById(SetNodeById { node_id })),
            },
            AColorSignal::AppendNode { group_id, node_id } => Self {
                signal: Some(Signal::AppendNode(AppendNode { group_id, node_id })),
            },
            AColorSignal::UpdateGroup { group_id } => Self {
                signal: Some(Signal::UpdateGroup(UpdateGroup { group_id })),
            },
            AColorSignal::EmptyGroup { group_id } => Self {
                signal: Some(Signal::EmptyGroup(EmptyGroup { group_id })),
            },
            AColorSignal::CoreChanged => Self {
                signal: Some(Signal::CoreChanged(CoreChanged {})),
            },
            AColorSignal::RuntimeValueChanged { key } => Self {
                signal: Some(Signal::RuntimeValueChanged(RuntimeValueChanged { key })),
            },
            AColorSignal::Shutdown => Self {
                signal: Some(Signal::Shutdown(Shutdown {})),
            },
            AColorSignal::SetApiStatus => Self {
                signal: Some(Signal::SetApiStatus(SetApiStatus {})),
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
                Signal::AppendGroup(m) => Self::AppendGroup {
                    group_id: m.group_id,
                },
                Signal::UpdateCoreStatus(_) => Self::UpdateCoreStatus,
                Signal::UpdateInbounds(_) => Self::UpdateInbounds,
                Signal::CoreConfigChanged(_) => Self::CoreConfigChanged,
                Signal::RemoveGroupById(m) => Self::RemoveGroupById {
                    group_id: m.group_id,
                },
                Signal::RemoveNodeById(m) => Self::RemoveNodeById { node_id: m.node_id },
                Signal::SetGroupById(m) => Self::SetGroupById {
                    group_id: m.group_id,
                },
                Signal::SetNodeById(m) => Self::SetNodeById { node_id: m.node_id },
                Signal::AppendNode(m) => Self::AppendNode {
                    group_id: m.group_id,
                    node_id: m.node_id,
                },
                Signal::UpdateGroup(m) => Self::UpdateGroup {
                    group_id: m.group_id,
                },
                Signal::EmptyGroup(m) => Self::EmptyGroup {
                    group_id: m.group_id,
                },
                Signal::CoreChanged(_) => Self::CoreChanged,
                Signal::RuntimeValueChanged(m) => Self::RuntimeValueChanged { key: m.key },
                Signal::Shutdown(_) => Self::Shutdown,
                Signal::SetApiStatus(_) => Self::SetApiStatus,
            })
            .unwrap_or(Self::Empty)
    }
}

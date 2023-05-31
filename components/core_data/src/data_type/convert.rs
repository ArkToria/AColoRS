impl From<crate::GroupData> for core_protobuf::acolors_proto::GroupData {
    fn from(group_data: crate::GroupData) -> Self {
        core_protobuf::acolors_proto::GroupData {
            id: group_data.id,
            name: group_data.name,
            is_subscription: group_data.is_subscription,
            group_type: group_data.group_type,
            url: group_data.url,
            cycle_time: group_data.cycle_time,
            created_at: group_data.created_at,
            modified_at: group_data.modified_at,
        }
    }
}
impl From<core_protobuf::acolors_proto::GroupData> for crate::GroupData {
    fn from(group_data: core_protobuf::acolors_proto::GroupData) -> Self {
        crate::GroupData {
            id: group_data.id,
            name: group_data.name,
            is_subscription: group_data.is_subscription,
            group_type: group_data.group_type,
            url: group_data.url,
            cycle_time: group_data.cycle_time,
            created_at: group_data.created_at,
            modified_at: group_data.modified_at,
        }
    }
}

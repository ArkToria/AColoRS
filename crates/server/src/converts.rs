impl From<profile_manager::GroupData> for crate::protobuf::acolors_proto::GroupData {
    fn from(group_data: profile_manager::GroupData) -> Self {
        crate::protobuf::acolors_proto::GroupData {
            id: group_data.id,
            name: group_data.name,
            is_subscription: group_data.is_subscription,
            group_type: group_data.group_type,
            url: group_data.url,
            cycle_time: group_data.cycle_time,
            create_at: group_data.create_at,
            modified_at: group_data.modified_at,
        }
    }
}
impl From<crate::protobuf::acolors_proto::GroupData> for profile_manager::GroupData {
    fn from(group_data: crate::protobuf::acolors_proto::GroupData) -> Self {
        profile_manager::GroupData {
            id: group_data.id,
            name: group_data.name,
            is_subscription: group_data.is_subscription,
            group_type: group_data.group_type,
            url: group_data.url,
            cycle_time: group_data.cycle_time,
            create_at: group_data.create_at,
            modified_at: group_data.modified_at,
        }
    }
}

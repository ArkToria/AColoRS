impl From<crate::GroupData> for core_protobuf::acolors_proto::GroupData {
    fn from(group_data: crate::GroupData) -> Self {
        core_protobuf::acolors_proto::GroupData {
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
impl From<core_protobuf::acolors_proto::GroupData> for crate::GroupData {
    fn from(group_data: core_protobuf::acolors_proto::GroupData) -> Self {
        crate::GroupData {
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

impl From<crate::NodeData> for core_protobuf::acolors_proto::NodeData {
    fn from(node_data: crate::NodeData) -> Self {
        core_protobuf::acolors_proto::NodeData {
            id: node_data.id,
            name: node_data.name,
            group_id: node_data.group_id,
            group_name: node_data.group_name,
            routing_id: node_data.routing_id,
            routing_name: node_data.routing_name,
            protocol: node_data.protocol,
            address: node_data.address,
            port: node_data.port,
            password: node_data.password,
            raw: node_data.raw,
            url: node_data.url,
            latency: node_data.latency,
            upload: node_data.upload,
            download: node_data.download,
            create_at: node_data.create_at,
            modified_at: node_data.modified_at,
        }
    }
}
impl From<core_protobuf::acolors_proto::NodeData> for crate::NodeData {
    fn from(node_data: core_protobuf::acolors_proto::NodeData) -> Self {
        crate::NodeData {
            id: node_data.id,
            name: node_data.name,
            group_id: node_data.group_id,
            group_name: node_data.group_name,
            routing_id: node_data.routing_id,
            routing_name: node_data.routing_name,
            protocol: node_data.protocol,
            address: node_data.address,
            port: node_data.port,
            password: node_data.password,
            raw: node_data.raw,
            url: node_data.url,
            latency: node_data.latency,
            upload: node_data.upload,
            download: node_data.download,
            create_at: node_data.create_at,
            modified_at: node_data.modified_at,
        }
    }
}

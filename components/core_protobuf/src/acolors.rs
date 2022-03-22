use utils::time::get_current_time;
pub mod acolors_proto {
    tonic::include_proto!("acolors");
}

impl acolors_proto::GroupData {
    pub fn update_modified_at(&mut self) {
        self.modified_at = get_current_time() as i64;
    }

    pub fn update_create_at(&mut self) {
        self.create_at = get_current_time() as i64;
    }
}

impl acolors_proto::NodeData {
    pub fn update_modified_at(&mut self) {
        self.modified_at = get_current_time() as i64;
    }

    pub fn update_create_at(&mut self) {
        self.create_at = get_current_time() as i64;
    }
    pub fn initialize(&mut self) {
        self.update_create_at();
        self.update_modified_at();
        self.latency = -1;
    }
}

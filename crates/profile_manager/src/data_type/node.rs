use utils::time::get_current_time;

#[derive(Debug)]
pub struct Node {
    pub id: i32,
    pub name: String,
    pub group_id: i32,
    pub group_name: String,
    pub routing_id: i32,
    pub routing_name: String,
    pub protocol: i32,
    pub address: String,
    pub port: u16,
    pub password: String,
    pub raw: String,
    pub url: String,
    pub latency: i32,
    pub upload: i64,
    pub download: i64,
    pub create_at: i64,
    pub modified_at: i64,
}

impl Node {
    pub fn update_modified_at(&mut self) {
        self.modified_at = get_current_time() as i64;
    }

    pub fn update_create_at(&mut self) {
        self.create_at = get_current_time() as i64;
    }
}

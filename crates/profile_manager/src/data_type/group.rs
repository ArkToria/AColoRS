use utils::time::get_current_time;

#[derive(Debug, Clone)]
pub struct Group {
    pub id: i32,
    pub name: String,
    pub is_subscription: bool,
    pub group_type: i32,
    pub url: String,
    pub cycle_time: i32,
    pub create_at: i64,
    pub modified_at: i64,
}

impl Group {
    pub fn update_modified_at(&mut self) {
        self.modified_at = get_current_time() as i64;
    }

    pub fn update_create_at(&mut self) {
        self.create_at = get_current_time() as i64;
    }
}

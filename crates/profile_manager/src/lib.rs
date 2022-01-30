mod test;

#[derive(Debug)]
struct Group {
    id: i32,
    name: String,
    is_subscription: bool,
    group_type: i32,
    url: String,
    cycle_time: i32,
    create_at: i64,
    modified_at: i64,
}

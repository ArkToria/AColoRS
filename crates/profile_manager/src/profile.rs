use crate::table_member::{grouplist::GroupList, runtime::RuntimeValue};

pub struct Profile {
    pub group_list: GroupList,
    pub runtime_value: RuntimeValue,
}

mod core;
pub mod v2ray;

pub use crate::core::CoreTool;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

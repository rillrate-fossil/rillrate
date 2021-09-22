use once_cell::sync::OnceCell;

pub static PRESERVED: OnceCell<Vec<u8>> = OnceCell::new();

#[macro_export]
macro_rules! embed_config {
    () => {
        let value = include_bytes!(env!("RR_CONFIG"));
        $crate::preserved::PRESERVED.set(value.to_vec());
    };
}

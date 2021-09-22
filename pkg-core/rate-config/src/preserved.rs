use once_cell::sync::OnceCell;

pub static PRESERVED: OnceCell<Vec<u8>> = OnceCell::new();

#[macro_export]
macro_rules! log {
    ($msg:expr) => {
        RILL.log($msg);
    };
}

#[macro_export]
macro_rules! attach_logger {
    () => {
        pub static RILL: rill::ProviderCell = rill::ProviderCell::new(std::module_path!());
    };
}

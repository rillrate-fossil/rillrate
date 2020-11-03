#[macro_export]
macro_rules! log {
    ($msg:expr) => {
        let wrapper = rill::provider::StaticJointWrapper::from(&RILL);
        if rill::provider::Joint::provider(&wrapper).is_active() {
            RILL.log($msg);
        }
    };
}

#[macro_export]
macro_rules! attach_logger {
    () => {
        pub static RILL: rill::StaticJoint = rill::StaticJoint::new(std::module_path!());
    };
}

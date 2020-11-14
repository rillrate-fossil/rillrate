// TODO: Imports here instad of the root module?

// TODO: Use `$crate` instead?

#[macro_export]
macro_rules! provider {
    () => {
        pub static RILL: $crate::Lazy<$crate::Provider> = $crate::Lazy::new(|| {
            let name = std::module_path!();
            let entry_id = $crate::EntryId::from(name);
            $crate::Provider::new(entry_id)
        });
    };
}

#[macro_export]
macro_rules! log {
    ($msg:expr) => {{
        {
            let rill = $crate::Lazy::force(&RILL);
            if rill.is_active() {
                rill.log($msg);
            }
        }
    }};
}

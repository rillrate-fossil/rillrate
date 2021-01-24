//! Macros to create static tracers.

// TODO: Imports here instad of the root module?

pub use crate::tracers::LogTracer;
pub use once_cell::sync::Lazy;
pub use rill_protocol::provider::{EntryId, Path};

#[doc(hidden)]
pub fn split_module_path(module_path: &'static str) -> Path {
    let path: Vec<_> = module_path.split("::").map(EntryId::from).collect();
    Path::from(path)
}

/// Creates staic provider.
#[macro_export]
macro_rules! provider {
    () => {
        $crate::provider!(@public false, "");
    };
    (public $info:expr) => {
        $crate::provider!(@public true, $info);
    };
    (@public $public:expr, $info:expr) => {
        pub static RILL: $crate::macros::Lazy<$crate::macros::LogTracer> =
            $crate::macros::Lazy::new(|| {
                let name = std::module_path!();
                let path = $crate::macros::split_module_path(name);
                let provider = $crate::macros::LogTracer::new(path);
                if $public {
                    provider.export($info);
                }
                provider
            });
    };
}

// TODO: Remove
/// Writes text to the global provider.
#[macro_export]
macro_rules! log {
    ($msg:expr) => {{
        {
            let rill = $crate::macros::Lazy::force(&RILL);
            if rill.is_active() {
                rill.log($msg, None);
            }
        }
    }};
}

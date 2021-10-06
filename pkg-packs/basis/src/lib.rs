pub mod frames;
pub mod manifest;
pub mod paths;

#[cfg(feature = "engine")]
pub use manifest::BindedTracer;
pub use paths::*;

#[cfg(feature = "engine")]
pub fn init() {
    // TODO: How to do it better?
    let _ = &*crate::manifest::paths::global::PATHS;
    let _ = &*crate::manifest::layouts::global::LAYOUTS;
}

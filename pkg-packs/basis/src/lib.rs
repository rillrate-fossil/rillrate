pub mod frames;
pub mod manifest;
pub mod paths;

#[cfg(feature = "engine")]
pub use manifest::BindedTracer;
pub use paths::*;

#[cfg(feature = "engine")]
pub fn init() {
    // TODO: How to do it better?
    drop(&*crate::manifest::paths::global::PATHS);
    drop(&*crate::manifest::layouts::global::LAYOUTS);
}

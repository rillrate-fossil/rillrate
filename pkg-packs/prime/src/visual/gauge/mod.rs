//! Value in a range.
//!
//! <img src="https://docs.rillrate.com/images/packs/prime/flows/gauge.gif" height="120px" />

pub mod state;
pub use state::*;

#[cfg(feature = "engine")]
pub mod tracer;
#[cfg(feature = "engine")]
pub use tracer::*;

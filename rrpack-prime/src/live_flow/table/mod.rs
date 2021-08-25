//! Table with rows and cols.
//!
//! <img src="https://docs.rillrate.com/images/packs/prime/flows/table.gif" height="120px" />

pub mod state;
pub use state::*;

#[cfg(feature = "engine")]
pub mod tracer;
#[cfg(feature = "engine")]
pub use tracer::*;

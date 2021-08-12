//! Live data streams tracks time automatically and used to provide live data.

pub mod board;
#[cfg(feature = "engine")]
pub use board::Board;

pub mod counter;
#[cfg(feature = "engine")]
pub use counter::Counter;

pub mod pulse;
#[cfg(feature = "engine")]
pub use pulse::Pulse;

pub mod auto_path;

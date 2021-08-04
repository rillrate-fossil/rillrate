//! Live data streams tracks time automatically and used to provide live data.

pub mod counter;
#[cfg(feature = "engine")]
pub use counter::CounterStatTracer;

pub mod pulse;
#[cfg(feature = "engine")]
pub use pulse::PulseFrameTracer;

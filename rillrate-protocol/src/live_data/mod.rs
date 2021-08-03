//! Live data streams tracks time automatically and used to provide live data.

pub mod counter;
pub use counter::CounterStatTracer;

pub mod pulse;
pub use pulse::PulseFrameTracer;

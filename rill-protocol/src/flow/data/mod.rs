//! Data Flows consists of three types of elements:
//! 1. `Flow` - immutable parameters of a data flow.
//! Flow is serialized and transferred with a description.
//! 2. `State` - mutable snapshot that contains all applied deltas and events.
//! It sent serialized on the beggining of Push mode or periodically in Push mode.
//! 3. `Event` - immutable separate change that has to be applied to the `State`.

pub mod counter;
pub use counter::CounterFlow;

pub mod dict;
pub use dict::DictFlow;

pub mod gauge;
pub use gauge::GaugeFlow;

pub mod histogram;
pub use histogram::HistogramFlow;

pub mod logger;
pub use logger::LoggerFlow;

pub mod pulse;
pub use pulse::PulseFlow;

pub mod table;
pub use table::TableFlow;

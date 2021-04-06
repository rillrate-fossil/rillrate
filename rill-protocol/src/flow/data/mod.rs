//! Data Flows consists of three types of elements:
//! 1. `Flow` - immutable parameters of a data flow.
//! Flow is serialized and transferred with a description.
//! 2. `State` - mutable snapshot that contains all applied deltas and events.
//! It sent serialized on the beggining of Push mode or periodically in Push mode.
//! 3. `Event` - immutable separate change that has to be applied to the `State`.

pub mod counter;
pub mod dict;
pub mod gauge;
pub mod histogram;
pub mod logger;
pub mod pulse;
pub mod table;

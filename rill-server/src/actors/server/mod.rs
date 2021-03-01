//! Standalone node that can be embedded into an app.

mod actor;
pub use actor::RillServer;

mod link;
pub use link::ServerLink;

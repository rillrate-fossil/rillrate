pub mod alert;
pub mod connection;
pub mod entry;
pub mod path;
pub mod ready_board;

/// `MetaFlow`s are flows that don't require getting a `Flow` object
/// to bootstrap a `State`.
pub trait MetaFlow: Default {}

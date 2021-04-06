//! Rill crate.

#![warn(missing_docs)]

mod actors;
pub mod config;
mod state;
pub mod tracers;
pub mod watchers;

metacrate::meta!();

pub use actors::engine::RillEngine;
pub use config::EngineConfig;

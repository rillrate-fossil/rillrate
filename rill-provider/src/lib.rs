//! Rill crate.

#![warn(missing_docs)]

mod actors;
pub mod config;
pub mod prelude;
mod state;
pub mod tracers;

metacrate::meta!();

pub use crate::actors::provider::RillProvider;

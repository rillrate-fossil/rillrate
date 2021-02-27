//! Rill crate.

#![warn(missing_docs)]

mod actors;
pub mod config;
mod state;
pub mod tracers;

metacrate::meta!();

pub use crate::actors::engine::RillEngine;

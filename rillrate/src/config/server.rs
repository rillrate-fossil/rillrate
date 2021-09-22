//! This module contains the main config.

use rill_config::{Config, ReadableConfig};
use rate_core::actors::node::NodeConfig;
use serde::Deserialize;

/// The main config struct.
#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct RillRateConfig {
    pub node: Option<NodeConfig>,
    /// Show explorer
    pub explorer: Option<bool>,
}

impl Config for RillRateConfig {}

impl ReadableConfig for RillRateConfig {}

use rate_config::{Config, ReadableConfig};
use rate_core::actors::node::NodeConfig;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct RillRateConfig {
    pub node: Option<NodeConfig>,
    /// Show explorer
    pub explorer: Option<bool>,
}

impl Config for RillRateConfig {}

impl ReadableConfig for RillRateConfig {}

use rate_config::{Config, ReadableConfig};
use rate_core::actors::node::NodeConfig;
// TODO: Don't use `Layout` type directly.
use rrpack_prime::manifest::layouts::layout::Layout;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct RillRateConfig {
    pub node: NodeConfig,
    /// Show explorer
    pub explorer: bool,
    pub layout: Vec<Layout>,
}

impl Config for RillRateConfig {}

impl ReadableConfig for RillRateConfig {}

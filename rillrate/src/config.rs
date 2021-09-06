use rate_config::{Config, ReadableConfig};
use rate_core::actors::node::NodeConfig;
// TODO: Don't use `Layout` type directly.
use rrpack_prime::manifest::layouts::layout::Layout;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct RillRateConfig {
    pub node: Option<NodeConfig>,
    /// Show explorer
    pub explorer: Option<bool>,
    pub layout: Option<Vec<Layout>>,
}

impl Config for RillRateConfig {}

impl ReadableConfig for RillRateConfig {}

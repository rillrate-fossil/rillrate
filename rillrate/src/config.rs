use rate_config::{Config, ReadableConfig};
use rate_core::actors::node::NodeConfig;
// TODO: Don't use `Layout` type directly.
use rill_protocol::io::provider::EntryId;
use rrpack_prime::manifest::layouts::layout::LayoutConfig;
use serde::{Deserialize};
use std::collections::HashMap;

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
pub struct RillRateConfig {
    pub node: Option<NodeConfig>,
    /// Show explorer
    pub explorer: Option<bool>,
    pub layout: HashMap<EntryId, LayoutConfig>,
}

impl Config for RillRateConfig {}

impl ReadableConfig for RillRateConfig {}

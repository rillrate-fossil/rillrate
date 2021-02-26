//! Configuration structs for the provider and tracers

use rill_protocol::config::ConfigPatch;
use rill_protocol::provider::EntryId;
use serde::Deserialize;
use std::env;

/// The external user app can set this value to override default server.
/// If embedded server started it can put its socket address here.
pub static NODE: ConfigPatch<String> = ConfigPatch::new("RILLRATE_NODE");

/// The external user app can set this value to override the default name.
pub static NAME: ConfigPatch<String> = ConfigPatch::new("RILLRATE_NAME");

/// Provider configuration
#[derive(Deserialize, Debug, Clone)]
pub struct ProviderConfig {
    // TODO: Use default serde value instead
    /// Node where connect the provider
    pub node: Option<String>,
    // TODO: Use default serde value instead
    /// The name of the provider
    pub name: Option<EntryId>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            node: None,
            name: None,
        }
    }
}

impl ProviderConfig {
    /// Returns `true` if node explicitly specified.
    pub fn is_node_specified(&self) -> bool {
        NODE.env_var().is_ok() || self.node.is_some()
    }

    /// Full url of the node
    pub fn node_url(&self) -> String {
        let host = env::var(ENV_NODE)
            // 1. Check the env var
            .ok()
            // 2. Check the config file
            .or_else(|| self.node.clone())
            // 3. Check the overriden default (if set)
            .or_else(|| DEFAULT_NODE.get().cloned())
            // 4. Use the default value
            .unwrap_or_else(|| "localhost:1636".into());
        format!("ws://{}/live/provider", host)
    }

    /// Name of the provider
    pub fn provider_name(&self) -> EntryId {
        env::var(ENV_NAME)
            // 1. Check the env var
            .ok()
            .map(EntryId::from)
            // 2. Check the config file
            .or_else(|| self.name.clone())
            // 3. Check the overriden default (if set)
            .or_else(|| DEFAULT_NAME.get().cloned())
            // 4. Use the default value
            .unwrap_or_else(|| "rillrate".into())
    }
}

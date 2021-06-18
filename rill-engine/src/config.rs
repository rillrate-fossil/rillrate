//! Configuration structs for the provider and tracers

use rill_protocol::config::ConfigPatch;
use rill_protocol::io::provider::{EntryId, ProviderType};
use serde::Deserialize;

/// The external user app can set this value to override default server.
/// If embedded server started it can put its socket address here.
pub static NODE: ConfigPatch<String> = ConfigPatch::new("RILLRATE_NODE");

/// The external user app can set this value to override the default name.
pub static NAME: ConfigPatch<EntryId> = ConfigPatch::new("RILLRATE_NAME");

/// Provider configuration
#[derive(Deserialize, Debug, Clone)]
pub struct EngineConfig {
    // TODO: Use default serde value instead
    /// Node where connect the provider
    pub node: Option<String>,
    // TODO: Use default serde value instead
    /// The name of the provider
    pub name: Option<EntryId>,
    /// The type of the provider
    pub provider_type: Option<ProviderType>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            node: None,
            name: None,
            provider_type: None,
        }
    }
}

impl EngineConfig {
    /// Returns `true` if node explicitly specified.
    pub fn is_node_specified(&self) -> bool {
        NODE.env_var().transpose().is_some() || self.node.is_some()
    }

    /// Full url of the node
    pub fn node_url(&self) -> String {
        let host = NODE.get(|| self.node.clone(), || "localhost:1636".into());
        format!("ws://{}/live/provider", host)
    }

    /// Name of the provider
    pub fn provider_name(&self) -> EntryId {
        NAME.get(
            || self.name.clone(),
            || {
                std::env::current_exe()
                    .ok()
                    .as_ref()
                    .and_then(|path| path.as_path().file_name())
                    .and_then(|path| path.to_str().map(EntryId::from))
                    .unwrap_or_else(|| "rillrate".into())
            },
        )
    }

    /// The type of the provider
    pub fn provider_type(&self) -> ProviderType {
        self.provider_type
            .clone()
            .unwrap_or_else(|| "generic".into())
    }
}

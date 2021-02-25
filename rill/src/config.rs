//! Configuration structs for the provider and tracers

use rill_protocol::provider::EntryId;
use serde::Deserialize;

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
    /// Full url of the node
    pub fn node_url(&self) -> String {
        let host = self
            .node
            .as_ref()
            .map(String::as_ref)
            .unwrap_or("localhost:1636");
        format!("ws://{}/live/provider", host)
    }

    /// Name of the provider
    pub fn provider_name(&self) -> EntryId {
        self.name
            .clone()
            .unwrap_or_else(|| EntryId::from("rillrate"))
    }
}

/*
impl ProviderConfig {
    pub fn new(host: String, entry_id: EntryId) -> Self {
        let url = format!("ws://{}/live/provider", host);
        let inner = Arc::new(RillConfigInner { entry_id, url });
        Self { inner }
    }

    pub fn entry_id(&self) -> &EntryId {
        &self.inner.entry_id
    }

    pub fn url(&self) -> &str {
        &self.inner.url
    }
}

#[derive(Debug)]
struct RillConfigInner {
}
*/

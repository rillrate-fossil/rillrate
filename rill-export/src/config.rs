use rill_protocol::config::ConfigPatch;
use serde::Deserialize;

// TODO: Support no env vars for `ConfigPatch`
pub static NODE: ConfigPatch<String> = ConfigPatch::new("VAR-NOT-SPECIFIED");

/// Exporter configuration
#[derive(Deserialize, Debug, Clone)]
pub struct ExportConfig {}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {}
    }
}

impl ExportConfig {
    /// Full url of the node
    pub fn node_url(&self) -> String {
        let host = NODE.get(|| None, || "localhost:9090".into());
        format!("ws://{}/live/client", host)
    }
}

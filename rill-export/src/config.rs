use rill_protocol::config::ConfigPatch;
use rill_protocol::provider::PathPattern;
use serde::Deserialize;
use std::collections::HashSet;

// TODO: Support no env vars for `ConfigPatch`
pub static NODE: ConfigPatch<String> = ConfigPatch::new("VAR-NOT-SPECIFIED");

///
/// Config of exporters.
#[derive(Deserialize)]
pub struct ExportConfig {
    /// Optional config for Prometheus
    pub prometheus: Option<PrometheusConfig>,
    /// Optional config for Graphite
    pub graphite: Option<GraphiteConfig>,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            prometheus: None,
            graphite: None,
        }
    }
}

impl ExportConfig {
    // TODO: Use direct connections
    /// Full url of the node
    pub fn node_url(&self) -> String {
        let host = NODE.get(|| None, || "localhost:9090".into());
        format!("ws://{}/live/client", host)
    }
}

/// Prometheus exporter config.
#[derive(Deserialize)]
pub struct PrometheusConfig {
    // TODO: Deserialize paths here directly using `FromStr`
    /// Patterns of paths.
    pub paths: HashSet<PathPattern>,
}

/// Graphite exporter config.
#[derive(Deserialize)]
pub struct GraphiteConfig {
    // TODO: Deserialize paths here directly using `FromStr`
    /// Patterns of paths.
    pub paths: HashSet<PathPattern>,
    /// Interval of uploading the data to the server.
    pub interval: Option<u64>,
}

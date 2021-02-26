//! The module contains all configuration structs for the embedded node.

use rill_protocol::config::ConfigPatch;
use rill_protocol::provider::PathPattern;
use serde::Deserialize;
use std::collections::HashSet;
use std::net::IpAddr;

/// Overrides default embedded server address.
pub static SERVER_ADDRESS: ConfigPatch<IpAddr> = ConfigPatch::new("RILLRATE_SERVER_ADDRESS");

/// Embedded server configuration.
#[derive(Deserialize)]
pub struct ServerConfig {
    /// An address where bind the server.
    pub address: Option<IpAddr>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self { address: None }
    }
}

impl ServerConfig {
    /// Returns address where bind a server
    pub fn server_address(&self) -> IpAddr {
        SERVER_ADDRESS.get(|| self.address.clone(), || "127.0.0.1".parse().unwrap())
    }
}

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

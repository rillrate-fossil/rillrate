use rill_protocol::provider::PathPattern;
use serde::Deserialize;
use std::collections::HashSet;
use std::net::IpAddr;

#[derive(Deserialize)]
pub struct ServerConfig {
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
        self.address
            .clone()
            // TODO: Don't parse
            .unwrap_or_else(|| "127.0.0.1".parse().unwrap())
    }
}

#[derive(Deserialize)]
pub struct ExportConfig {
    pub prometheus: Option<PrometheusConfig>,
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

#[derive(Deserialize)]
pub struct PrometheusConfig {
    // TODO: Deserialize paths here directly using `FromStr`
    pub paths: HashSet<PathPattern>,
}

#[derive(Deserialize)]
pub struct GraphiteConfig {
    // TODO: Deserialize paths here directly using `FromStr`
    pub paths: HashSet<PathPattern>,
    pub interval: Option<u64>,
}

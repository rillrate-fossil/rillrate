//! The module contains all configuration structs for the embedded node.

use once_cell::sync::OnceCell;
use rill_protocol::provider::PathPattern;
use serde::Deserialize;
use std::collections::HashSet;
use std::env;
use std::net::IpAddr;

/// Overrides default embedded server address.
pub static DEFAULT_SERVER_ADDRESS: OnceCell<IpAddr> = OnceCell::new();
const ENV_SERVER_ADDRESS: &str = "RILLRATE_SERVER_ADDRESS";

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
        env::var(ENV_SERVER_ADDRESS)
            // 1. Check the env var
            .ok()
            .and_then(|addr| {
                addr.parse()
                    .map_err(|err| {
                        log::error!("Can't parse embedded server address from env var: {}", err);
                    })
                    .ok()
            })
            // 2. Check the config file
            .or_else(|| self.address.clone())
            // 3. Check the overriden default (if set)
            .or_else(|| DEFAULT_SERVER_ADDRESS.get().cloned())
            // 4. Use the default value
            // TODO: Don't use parse here
            .unwrap_or_else(|| "127.0.0.1".parse().unwrap())
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

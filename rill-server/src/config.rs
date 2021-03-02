//! The module contains all configuration structs for the embedded node.

use rill_protocol::config::ConfigPatch;
use serde::Deserialize;
use std::net::IpAddr;

/// Overrides default embedded server address.
pub static SERVER_ADDRESS: ConfigPatch<IpAddr> = ConfigPatch::new("RILLRATE_SERVER_ADDRESS");

/// Embedded server configuration.
#[derive(Deserialize, Debug)]
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
        SERVER_ADDRESS.get(|| self.address, || "127.0.0.1".parse().unwrap())
    }
}

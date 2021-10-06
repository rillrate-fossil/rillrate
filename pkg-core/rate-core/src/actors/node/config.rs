use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Server configuration.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NodeConfig {
    /// An address where bind the server.
    pub external_address: Option<SocketAddr>,
    pub internal_address: Option<SocketAddr>,
}

#[allow(clippy::derivable_impls)]
impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            external_address: None,
            internal_address: None,
        }
    }
}

impl NodeConfig {
    pub fn external_address(&self) -> SocketAddr {
        self.external_address
            .unwrap_or_else(|| "0.0.0.0:6361".parse().unwrap())
    }

    pub fn internal_address(&self) -> SocketAddr {
        self.internal_address
            .unwrap_or_else(|| "127.0.0.1:1636".parse().unwrap())
    }
}

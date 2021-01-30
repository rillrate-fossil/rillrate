use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::LiteTask;
use rill_protocol::provider::Path;
use serde::{de, Deserialize, Deserializer};
use std::collections::HashSet;
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathPattern {
    pub path: Path,
}

impl<'de> Deserialize<'de> for PathPattern {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let path: Path = FromStr::from_str(&s).map_err(de::Error::custom)?;
        Ok(PathPattern { path })
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub server: Option<ServerConfig>,
    pub export: Option<ExportConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: None,
            export: None,
        }
    }
}

impl Config {
    /// Returns address where bind a server
    pub fn server_address(&self) -> IpAddr {
        self.server
            .as_ref()
            .and_then(|server| server.address.clone())
            .unwrap_or_else(|| "127.0.0.1".parse().unwrap())
    }

    /// Activate `metadata` providers
    pub fn meta(&self) -> bool {
        self.server
            .as_ref()
            .and_then(|server| server.meta.clone())
            .unwrap_or(false)
    }
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub address: Option<IpAddr>,
    pub meta: Option<bool>,
}

#[derive(Deserialize)]
pub struct ExportConfig {
    pub prometheus: Option<PrometheusConfig>,
    pub graphite: Option<GraphiteConfig>,
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

pub struct ReadConfigFile(pub PathBuf);

#[async_trait]
impl LiteTask for ReadConfigFile {
    type Output = Config;

    async fn interruptable_routine(mut self) -> Result<Self::Output, Error> {
        let mut file = File::open(self.0).await?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;
        let config: Config = toml::from_slice(&contents)?;
        Ok(config)
    }
}

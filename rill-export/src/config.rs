use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::LiteTask;
use rill_protocol::provider::PathPattern;
use serde::Deserialize;
use std::collections::HashSet;
use std::net::IpAddr;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

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
    // TODO: Move to own `config-utils` crate?
    async fn read(path: PathBuf) -> Result<Self, Error> {
        let mut file = File::open(path).await?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;
        let config: Config = toml::from_slice(&contents)?;
        Ok(config)
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

    /* IMPORTANT!!! Metas has to be activated for every module manually
    /// Activate `metadata` providers
    pub fn meta(&self) -> bool {
        self.global
            .as_ref()
            .and_then(|server| server.meta.clone())
            .unwrap_or(false)
    }
    */
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub address: Option<IpAddr>,
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

// TODO: Dry it (move to `config-utils`)
pub struct ReadConfigFile(pub PathBuf);

#[async_trait]
impl LiteTask for ReadConfigFile {
    type Output = Config;

    async fn interruptable_routine(mut self) -> Result<Self::Output, Error> {
        Config::read(self.0).await
    }
}

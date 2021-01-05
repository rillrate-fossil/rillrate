use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::LiteTask;
use rill_protocol::provider::Path;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub export: ExportConfig,
}

#[derive(Serialize, Deserialize)]
pub struct ExportConfig {
    pub prometheus: Option<PrometheusConfig>,
    pub graphite: Option<GraphiteConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct PrometheusConfig {
    // TODO: Deserialize paths here directly using `FromStr`
    pub paths: HashSet<Path>,
}

#[derive(Serialize, Deserialize)]
pub struct GraphiteConfig {
    // TODO: Deserialize paths here directly using `FromStr`
    pub paths: HashSet<Path>,
    pub interval: Option<u64>,
}

pub struct ReadConfigFile;

#[async_trait]
impl LiteTask for ReadConfigFile {
    type Output = Config;

    async fn interruptable_routine(mut self) -> Result<Self::Output, Error> {
        let mut file = File::open(crate::env::config()).await?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;
        let config: Config = toml::from_slice(&contents)?;
        Ok(config)
    }
}

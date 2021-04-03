use anyhow::Error;
use async_trait::async_trait;
use meio::LiteTask;
use rill_engine::config::EngineConfig;
use rill_server::config::ServerConfig;
use serde::Deserialize;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

const DEF_CONFIG: &str = "rillrate.toml";

#[derive(Deserialize, Debug)]
pub struct Config {
    pub rillrate: Option<EngineConfig>,
    pub server: Option<ServerConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rillrate: None,
            server: None,
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

pub struct ReadConfigFile(pub Option<PathBuf>);

#[async_trait]
impl LiteTask for ReadConfigFile {
    type Output = Option<Config>;

    async fn interruptable_routine(mut self) -> Result<Self::Output, Error> {
        let config = {
            if let Some(path) = self.0 {
                Some(Config::read(path).await?)
            } else {
                let path = DEF_CONFIG.into();
                Config::read(path).await.ok()
            }
        };
        log::trace!("Config ready: {:?}", config);
        Ok(config)
    }
}

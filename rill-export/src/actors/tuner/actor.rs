use crate::actors::embedded_node::EmbeddedNode;
use crate::actors::exporter::ExporterLinkForCtrl;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    Actor, Context, IdOf, InterruptedBy, LiteTask, StartedBy, TaskEliminated, TaskError,
};
use rill::protocol::Path;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub struct Tuner {
    exporter: ExporterLinkForCtrl,
}

impl Tuner {
    pub fn new(exporter: ExporterLinkForCtrl) -> Self {
        Self { exporter }
    }
}

impl Actor for Tuner {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<EmbeddedNode> for Tuner {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.spawn_task(ReadConfigFile, ());
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<EmbeddedNode> for Tuner {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<ReadConfigFile> for Tuner {
    async fn handle(
        &mut self,
        _id: IdOf<ReadConfigFile>,
        result: Result<Config, TaskError>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        match result {
            Ok(mut config) => {
                if let Some(export) = config.export.take() {
                    for path_str in export {
                        let path: Path = path_str.parse()?;
                        log::info!("Export path: {}", path);
                        self.exporter.export_path(path).await?;
                    }
                }
            }
            Err(err) => {
                log::warn!(
                    "Can't read config file. No special configuration parameters applied: {}",
                    err
                );
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct Config {
    // TODO: Deserialize paths here directly using `FromStr`
    export: Option<Vec<String>>,
}

struct ReadConfigFile;

#[async_trait]
impl LiteTask for ReadConfigFile {
    type Output = Config;

    async fn interruptable_routine(mut self) -> Result<Self::Output, Error> {
        let config_path = std::env::var("RILL_CONFIG").unwrap_or_else(|_err| "rill.toml".into());
        let mut file = File::open(config_path).await?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;
        let config: Config = toml::from_slice(&contents)?;
        Ok(config)
    }
}

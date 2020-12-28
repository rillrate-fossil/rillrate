use crate::actors::embedded_node::EmbeddedNode;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    Action, ActionHandler, Actor, Context, IdOf, InterruptedBy, LiteTask, StartedBy,
    TaskEliminated, TaskError,
};
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub struct Tuner {}

impl Tuner {
    pub fn new() -> Self {
        Self {}
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
            Ok(config) => for path_to_export in config.export {},
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
    export: Vec<String>,
}

struct ReadConfigFile;

#[async_trait]
impl LiteTask for ReadConfigFile {
    type Output = Config;

    async fn interruptable_routine(mut self) -> Result<Self::Output, Error> {
        let mut file = File::open("rill.toml").await?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;
        let config: Config = toml::from_slice(&contents)?;
        Ok(config)
    }
}

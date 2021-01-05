use crate::actors::embedded_node::EmbeddedNode;
use crate::actors::exporter::{publishers, ExporterLinkForClient};
use crate::config::{Config, ReadConfigFile};
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{Actor, Context, IdOf, InterruptedBy, StartedBy, TaskEliminated, TaskError};
use rill_protocol::provider::Path;

pub struct Tuner {
    exporter: ExporterLinkForClient,
}

impl Tuner {
    pub fn new(exporter: ExporterLinkForClient) -> Self {
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
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        match result {
            Ok(mut config) => {
                if let Some(config) = config.export.prometheus.take() {
                    self.exporter
                        .start_publisher::<publishers::PrometheusExporter>(config)
                        .await?;
                }
                if let Some(config) = config.export.graphite.take() {
                    self.exporter
                        .start_publisher::<publishers::GraphiteExporter>(config)
                        .await?;
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

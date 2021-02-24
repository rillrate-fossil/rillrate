use crate::actors::exporter::Exporter;
use crate::actors::exporter::{publishers, ExporterLinkForClient};
use crate::actors::server::Server;
use crate::config::{Config, ReadConfigFile};
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    Actor, Context, Eliminated, IdOf, InterruptedBy, StartedBy, System, TaskEliminated, TaskError,
};
use meio_connect::server::HttpServer;
use std::path::PathBuf;

pub struct EmbeddedNode {
    config_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    Tuning,
    Exporter,
    HttpServer,
    Endpoints,
}

impl Actor for EmbeddedNode {
    type GroupBy = Group;
}

impl EmbeddedNode {
    pub fn new(config_path: Option<PathBuf>) -> Self {
        Self { config_path }
    }
}

#[async_trait]
impl StartedBy<System> for EmbeddedNode {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![
            Group::Tuning,
            Group::Exporter,
            Group::HttpServer,
            Group::Endpoints,
        ]);
        if let Some(config_path) = self.config_path.clone() {
            let config_task = ReadConfigFile(config_path);
            ctx.spawn_task(config_task, Group::Tuning);
        } else {
            log::info!("No config file provided. Default settings will be used.");
        }
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<System> for EmbeddedNode {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<ReadConfigFile> for EmbeddedNode {
    async fn handle(
        &mut self,
        _id: IdOf<ReadConfigFile>,
        result: Result<Config, TaskError>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let mut config = result
            .map_err(|err| {
                log::warn!(
                    "Can't read config file. No special configuration parameters applied: {}",
                    err
                );
            })
            .unwrap_or_default();

        // Starting all basic actors
        // TODO: Don't parse it
        let extern_addr = format!("{}:{}", config.server_address(), 9090).parse()?;
        let extern_http_server_actor = HttpServer::new(extern_addr);
        let extern_http_server = ctx.spawn_actor(extern_http_server_actor, Group::HttpServer);

        // TODO: Don't parse it
        let inner_addr = format!("127.0.0.1:{}", rill_protocol::PORT.get()).parse()?;
        let inner_http_server_actor = HttpServer::new(inner_addr);
        let inner_http_server = ctx.spawn_actor(inner_http_server_actor, Group::HttpServer);

        let exporter_actor = Exporter::new(extern_http_server.link());
        let exporter = ctx.spawn_actor(exporter_actor, Group::Exporter);

        let server_actor = Server::new(
            inner_http_server.link(),
            extern_http_server.link(),
            exporter.link(),
        );
        let _server = ctx.spawn_actor(server_actor, Group::Endpoints);

        let mut exporter: ExporterLinkForClient = exporter.link();

        // Spawn exporters if they are exist
        if let Some(mut export) = config.export.take() {
            if let Some(config) = export.prometheus.take() {
                exporter
                    .start_publisher::<publishers::PrometheusPublisher>(config)
                    .await?;
            }
            if let Some(config) = export.graphite.take() {
                exporter
                    .start_publisher::<publishers::GraphitePublisher>(config)
                    .await?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Eliminated<Exporter> for EmbeddedNode {
    async fn handle(&mut self, _id: IdOf<Exporter>, _ctx: &mut Context<Self>) -> Result<(), Error> {
        log::info!("Exporter finished");
        Ok(())
    }
}

#[async_trait]
impl Eliminated<HttpServer> for EmbeddedNode {
    async fn handle(
        &mut self,
        _id: IdOf<HttpServer>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::info!("HttpServer finished");
        Ok(())
    }
}

#[async_trait]
impl Eliminated<Server> for EmbeddedNode {
    async fn handle(&mut self, _id: IdOf<Server>, _ctx: &mut Context<Self>) -> Result<(), Error> {
        log::info!("Server finished");
        Ok(())
    }
}

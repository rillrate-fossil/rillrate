use crate::actors::exporter::Exporter;
use crate::actors::exporter::{publishers, ExporterLinkForClient};
use crate::actors::server::Server;
use crate::config::{ExportConfig, ServerConfig};
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    Actor, Context, Eliminated, IdOf, InterruptedBy, StartedBy, TaskEliminated, TaskError,
};
use meio_connect::server::HttpServer;

/// Embedded node.
pub struct EmbeddedNode {
    server_config: ServerConfig,
    export_config: ExportConfig,
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
    /// Create a new instance of an embedded node.
    pub fn new(server_config: Option<ServerConfig>, export_config: Option<ExportConfig>) -> Self {
        Self {
            server_config: server_config.unwrap_or_default(),
            export_config: export_config.unwrap_or_default(),
        }
    }
}

#[async_trait]
impl<T: Actor> StartedBy<T> for EmbeddedNode {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![
            Group::Tuning,
            Group::Exporter,
            Group::HttpServer,
            Group::Endpoints,
        ]);

        // Starting all basic actors
        // TODO: Don't parse it
        let watcher = crate::EXTERN_ADDR.lock().await.0.take();
        // TODO: Use port from a config here
        let extern_addr = format!("{}:{}", self.server_config.server_address(), 9090).parse()?;
        let extern_http_server_actor = HttpServer::new(extern_addr, watcher);
        let extern_http_server = ctx.spawn_actor(extern_http_server_actor, Group::HttpServer);

        // TODO: Don't parse it
        let watcher = crate::INTERN_ADDR.lock().await.0.take();
        // TODO: Use port from config or `any` (0)
        let inner_addr = format!("127.0.0.1:{}", 0).parse()?;
        let inner_http_server_actor = HttpServer::new(inner_addr, watcher);
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
        if let Some(config) = self.export_config.prometheus.take() {
            exporter
                .start_publisher::<publishers::PrometheusPublisher>(config)
                .await?;
        }
        if let Some(config) = self.export_config.graphite.take() {
            exporter
                .start_publisher::<publishers::GraphitePublisher>(config)
                .await?;
        }

        Ok(())
    }
}

#[async_trait]
impl<T: Actor> InterruptedBy<T> for EmbeddedNode {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
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

use crate::actors::router::Router;
use crate::config::ServerConfig;
use anyhow::Error;
use async_trait::async_trait;
use meio::{Actor, Context, Eliminated, IdOf, InterruptedBy, StartedBy};
use meio_connect::server::HttpServer;
use rill_client::actors::broadcaster::Broadcaster;

/// Embedded node.
pub struct RillServer {
    server_config: ServerConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    Tuning,
    Broadcaster,
    HttpServer,
    Endpoints,
}

impl Actor for RillServer {
    type GroupBy = Group;
}

impl RillServer {
    /// Create a new instance of an embedded node.
    pub fn new(server_config: Option<ServerConfig>) -> Self {
        Self {
            server_config: server_config.unwrap_or_default(),
        }
    }
}

#[async_trait]
impl<T: Actor> StartedBy<T> for RillServer {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![
            Group::Tuning,
            Group::Broadcaster,
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

        let exporter_actor = Broadcaster::new();
        let exporter = ctx.spawn_actor(exporter_actor, Group::Broadcaster);

        let server_actor = Router::new(
            inner_http_server.link(),
            extern_http_server.link(),
            exporter.link(),
        );
        let _server = ctx.spawn_actor(server_actor, Group::Endpoints);

        /*
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
        */

        Ok(())
    }
}

#[async_trait]
impl<T: Actor> InterruptedBy<T> for RillServer {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl Eliminated<Broadcaster> for RillServer {
    async fn handle(
        &mut self,
        _id: IdOf<Broadcaster>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::info!("Broadcaster finished");
        Ok(())
    }
}

#[async_trait]
impl Eliminated<HttpServer> for RillServer {
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
impl Eliminated<Router> for RillServer {
    async fn handle(&mut self, _id: IdOf<Router>, _ctx: &mut Context<Self>) -> Result<(), Error> {
        log::info!("Router finished");
        Ok(())
    }
}

use crate::config::ExportConfig;
use crate::publishers::Publisher;
use anyhow::Error;
use async_trait::async_trait;
use meio::{Actor, Address, Context, Eliminated, IdOf, InterruptedBy, StartedBy};
use meio_connect::server::HttpServerLink;
use rill_client::actors::broadcaster::{Broadcaster, BroadcasterLinkForClient};
use rill_client::actors::client::RillClient;

pub struct RillExport {
    config: ExportConfig,
    client: Option<Address<RillClient>>,
    broadcaster: Option<Address<Broadcaster>>,
    /// It used to bind publishers that require to have an HTTP endpoint.
    /// Like `Prometheus` publisher.
    server: HttpServerLink,
}

impl RillExport {
    pub fn new(config: ExportConfig, server: HttpServerLink) -> Self {
        Self {
            config,
            client: None,
            broadcaster: None,
            server,
        }
    }

    fn get_broadcaster(&self) -> Result<BroadcasterLinkForClient, Error> {
        self.broadcaster
            .clone()
            .map(BroadcasterLinkForClient::from)
            .ok_or_else(|| Error::msg("No broadcaster attached to RillExport"))
    }

    fn spawn_publisher<T: Publisher>(&mut self, config: T::Config) -> Result<(), Error> {
        let broadcaster = self.get_broadcaster()?;
        let publihser = T::create(config, broadcaster, &self.server);
        Ok(())
    }
}

impl Actor for RillExport {
    type GroupBy = ();
}

#[async_trait]
impl<T: Actor> StartedBy<T> for RillExport {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let url = self.config.node_url();
        let actor = Broadcaster::new();
        let broadcaster = ctx.spawn_actor(actor, ());
        let link = broadcaster.link();
        self.broadcaster = Some(broadcaster);

        let actor = RillClient::new(url, link);
        let client = ctx.spawn_actor(actor, ());
        self.client = Some(client);

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
impl<T: Actor> InterruptedBy<T> for RillExport {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl Eliminated<Broadcaster> for RillExport {
    async fn handle(
        &mut self,
        _id: IdOf<Broadcaster>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl Eliminated<RillClient> for RillExport {
    async fn handle(
        &mut self,
        _id: IdOf<RillClient>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        Ok(())
    }
}

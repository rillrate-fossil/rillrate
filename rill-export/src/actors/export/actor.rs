use crate::config::ExportConfig;
use crate::publishers::{self, Publisher};
use anyhow::Error;
use async_trait::async_trait;
use meio::{Actor, Address, Context, Eliminated, IdOf, InterruptedBy, StartedBy};
use meio_connect::server::HttpServerLink;
use rill_client::actors::broadcaster::{Broadcaster, BroadcasterLinkForClient};
use rill_client::actors::client::{ClientLink, RillClient};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    Middleware,
    Publishers,
}

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

    fn get_client(&self) -> Result<ClientLink, Error> {
        self.client
            .clone()
            .map(ClientLink::from)
            .ok_or_else(|| Error::msg("No broadcaster attached to RillExport"))
    }

    fn spawn_publisher<T: Publisher>(
        &mut self,
        config: T::Config,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let broadcaster = self.get_broadcaster()?;
        let client = self.get_client()?;
        let publisher = T::create(config, broadcaster, client, &self.server);
        ctx.spawn_actor(publisher, Group::Publishers);
        Ok(())
    }
}

impl Actor for RillExport {
    type GroupBy = Group;
}

#[async_trait]
impl<T: Actor> StartedBy<T> for RillExport {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![Group::Publishers, Group::Middleware]);
        let url = self.config.node_url();
        let actor = Broadcaster::new();
        let broadcaster = ctx.spawn_actor(actor, Group::Middleware);
        let link = broadcaster.link();
        self.broadcaster = Some(broadcaster);

        let actor = RillClient::new(url, link);
        let client = ctx.spawn_actor(actor, Group::Middleware);
        self.client = Some(client);

        /*
        if let Some(config) = self.config.prometheus.take() {
            self
                .spawn_publisher::<publishers::PrometheusPublisher>(config)
                .await?;
        }
        */
        if let Some(config) = self.config.graphite.take() {
            self.spawn_publisher::<publishers::GraphitePublisher>(config, ctx)?;
        }

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

#[async_trait]
impl<T: Publisher> Eliminated<T> for RillExport {
    async fn handle(&mut self, _id: IdOf<T>, _ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}

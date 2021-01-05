use super::{link, ExportEvent, GraphiteExporter, PathNotification, PrometheusExporter};
use crate::actors::embedded_node::EmbeddedNode;
use crate::actors::provider_session::ProviderSessionLink;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    ActionHandler, ActionRecipient, Actor, Context, Distributor, Eliminated, Id, IdOf,
    InteractionHandler, InterruptedBy, StartedBy, TryConsumer,
};
use meio_connect::server::HttpServerLink;
use rill_protocol::provider::Path;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Reason {
    #[error("No active provider available")]
    NoActiveSession,
    #[error("No active exporters available")]
    NoExporters,
}

#[derive(Debug, Default)]
struct Record {
    distributor: Distributor<ExportEvent>,
    declared: bool,
}

/// The `Actor` that subscribes to data according to available `Path`s.
pub struct Exporter {
    server: HttpServerLink,
    provider: Option<ProviderSessionLink>,
    paths_trackers: Distributor<PathNotification>,
    recipients: HashMap<Path, Record>,
}

impl Exporter {
    pub fn new(server: HttpServerLink) -> Self {
        Self {
            server,
            provider: None,
            paths_trackers: Distributor::new(),
            recipients: HashMap::new(),
        }
    }

    fn provider(&mut self) -> Result<&mut ProviderSessionLink, Reason> {
        self.provider.as_mut().ok_or(Reason::NoActiveSession)
    }
}

impl Actor for Exporter {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<EmbeddedNode> for Exporter {
    async fn handle(&mut self, _ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<EmbeddedNode> for Exporter {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl Eliminated<GraphiteExporter> for Exporter {
    async fn handle(
        &mut self,
        _id: IdOf<GraphiteExporter>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::info!("GraphiteExporter finished");
        Ok(())
    }
}

#[async_trait]
impl Eliminated<PrometheusExporter> for Exporter {
    async fn handle(
        &mut self,
        _id: IdOf<PrometheusExporter>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::info!("PrometheusExporter finished");
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::SessionLifetime> for Exporter {
    async fn handle(
        &mut self,
        msg: link::SessionLifetime,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        use link::SessionLifetime::*;
        match msg {
            Attached { session } => {
                // Don't subscribe here till the stream (path) will be declared.
                self.provider = Some(session);
            }
            Detached => {
                self.provider.take();
                for record in self.recipients.values_mut() {
                    record.declared = false;
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::DataReceived> for Exporter {
    async fn handle(
        &mut self,
        msg: link::DataReceived,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let path = msg.path.clone();
        let event = ExportEvent::BroadcastData {
            path: msg.path,
            timestamp: msg.timestamp,
            data: msg.data,
        };
        let record = self.recipients.entry(path).or_default();
        record.distributor.act_all(event).await?;
        Ok(())
    }
}

impl Exporter {
    /*
    async fn begin_export(&mut self, path: Path) -> Result<(), Error> {
        let event = ExportEvent::SetInfo {
            path: path.clone(),
            info: "<todo>".into(),
        };
        self.broadcast(event)?;
        self.provider()?.subscribe(path).await?;
        Ok(())
    }
    */
}

#[async_trait]
impl ActionHandler<link::PathDeclared> for Exporter {
    async fn handle(
        &mut self,
        msg: link::PathDeclared,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let path = msg.description.path;
        let stream_type = msg.description.stream_type;
        log::info!("Declare path: {}", path);
        let record = self.recipients.entry(path.clone()).or_default();
        record.declared = true;
        let msg = PathNotification { path, stream_type };
        self.paths_trackers.act_all(msg).await?;
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::SubscribeToPaths> for Exporter {
    async fn handle(
        &mut self,
        msg: link::SubscribeToPaths,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        self.paths_trackers.insert(msg.recipient);
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::SubscribeToData> for Exporter {
    async fn handle(
        &mut self,
        msg: link::SubscribeToData,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let path = msg.path.clone();
        let record = self.recipients.entry(msg.path).or_default();
        record.distributor.insert(msg.recipient);
        if record.distributor.len() == 1 && record.declared {
            self.provider()?.subscribe(path).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::StartPrometheus> for Exporter {
    async fn handle(
        &mut self,
        msg: link::StartPrometheus,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let prometheus_actor =
            PrometheusExporter::new(msg.config, ctx.address().link(), self.server.clone());
        let prometheus = ctx.spawn_actor(prometheus_actor, ());
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::StartGraphite> for Exporter {
    async fn handle(
        &mut self,
        msg: link::StartGraphite,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let graphite_actor = GraphiteExporter::new(msg.config, ctx.address().link());
        let graphite = ctx.spawn_actor(graphite_actor, ());
        Ok(())
    }
}

/*
#[async_trait]
impl<A> ActionHandler<link::GraspExportStream<A>> for Exporter
where
    A: Actor + TryConsumer<ExportEvent, Error = broadcast::RecvError>,
{
    async fn handle(
        &mut self,
        msg: link::GraspExportStream<A>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let rx = self.sender.subscribe();
        msg.listener.attach(rx);
        Ok(())
    }
}

#[async_trait]
impl InteractionHandler<link::GetPaths> for Exporter {
    async fn handle(
        &mut self,
        _: link::GetPaths,
        _ctx: &mut Context<Self>,
    ) -> Result<HashSet<Path>, Error> {
        Ok(self.declared_paths.clone())
    }
}

#[async_trait]
impl InteractionHandler<link::GetProviderSession> for Exporter {
    async fn handle(
        &mut self,
        _: link::GetProviderSession,
        _ctx: &mut Context<Self>,
    ) -> Result<ProviderSessionLink, Error> {
        self.provider
            .clone()
            .ok_or(Reason::NoActiveSession)
            .map_err(Error::from)
    }
}
*/

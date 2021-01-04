use super::link;
use super::{ExportEvent, GraphiteExporter, PrometheusExporter};
use crate::actors::embedded_node::EmbeddedNode;
use crate::actors::provider_session::ProviderSessionLink;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    ActionHandler, Actor, Context, Eliminated, IdOf, InteractionHandler, InterruptedBy, StartedBy,
    TryConsumer,
};
use meio_connect::server::HttpServerLink;
use rill_protocol::provider::Path;
use std::collections::HashSet;
use thiserror::Error;
use tokio::sync::broadcast;

#[derive(Debug, Error)]
pub enum Reason {
    #[error("No active provider available")]
    NoActiveSession,
    #[error("No active exporters available")]
    NoExporters,
}

/// The `Actor` that subscribes to data according to available `Path`s.
pub struct Exporter {
    server: HttpServerLink,
    provider: Option<ProviderSessionLink>,
    paths_to_export: HashSet<Path>,
    declared_paths: HashSet<Path>,
    sender: broadcast::Sender<ExportEvent>,
}

impl Exporter {
    pub fn new(server: HttpServerLink) -> Self {
        let (sender, _) = broadcast::channel(32);
        Self {
            server,
            provider: None,
            paths_to_export: HashSet::new(),
            declared_paths: HashSet::new(),
            sender,
        }
    }

    fn provider(&mut self) -> Result<&mut ProviderSessionLink, Reason> {
        self.provider.as_mut().ok_or(Reason::NoActiveSession)
    }

    fn broadcast(&self, event: ExportEvent) -> Result<(), Error> {
        if self.sender.receiver_count() > 0 {
            self.sender.send(event).map_err(|_| Reason::NoExporters)?;
        }
        Ok(())
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
                self.provider = Some(session);
            }
            Detached => {
                self.provider.take();
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
        let event = ExportEvent::BroadcastData {
            path: msg.path,
            timestamp: msg.timestamp,
            data: msg.data,
        };
        self.broadcast(event)?;
        Ok(())
    }
}

impl Exporter {
    async fn begin_export(&mut self, path: Path) -> Result<(), Error> {
        let event = ExportEvent::SetInfo {
            path: path.clone(),
            info: "<todo>".into(),
        };
        self.broadcast(event)?;
        self.provider()?.subscribe(path).await?;
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::PathDeclared> for Exporter {
    async fn handle(
        &mut self,
        msg: link::PathDeclared,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let path = msg.description.path;
        log::info!("Declare path: {}", path);
        self.declared_paths.insert(path.clone());
        if self.paths_to_export.contains(&path) {
            self.begin_export(path).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::ExportPath> for Exporter {
    async fn handle(
        &mut self,
        msg: link::ExportPath,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let path = msg.path;
        self.paths_to_export.insert(path.clone());
        if self.declared_paths.contains(&path) {
            self.begin_export(path).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::StartPrometheus> for Exporter {
    async fn handle(
        &mut self,
        _msg: link::StartPrometheus,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let prometheus_actor = PrometheusExporter::new(self.server.clone());
        let prometheus = ctx.spawn_actor(prometheus_actor, ());
        let rx = self.sender.subscribe();
        prometheus.attach(rx);
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::StartGraphite> for Exporter {
    async fn handle(
        &mut self,
        _msg: link::StartGraphite,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let graphite_actor = GraphiteExporter::new();
        let graphite = ctx.spawn_actor(graphite_actor, ());
        let rx = self.sender.subscribe();
        graphite.attach(rx);
        Ok(())
    }
}

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

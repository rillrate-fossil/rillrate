use super::{link, ExportEvent, PathNotification, Publisher};
use crate::actors::embedded_node::EmbeddedNode;
use crate::actors::provider_session::ProviderSessionLink;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    ActionHandler, Actor, Context, Distributor, Eliminated, IdOf, InterruptedBy, StartedBy,
};
use meio_connect::server::HttpServerLink;
use rill_protocol::provider::{Description, Path};
use std::collections::{hash_map::Entry, HashMap};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Reason {
    #[error("No active provider available")]
    NoActiveSession,
    #[error("No active exporters available")]
    NoExporters,
    #[error("Path already declared {0}")]
    AlreadyDeclaredPath(Path),
    #[error("Path was not declared {0}")]
    NotDeclaredPath(Path),
    #[error("No meta for path {0}")]
    NoMetaForPath(Path),
}

#[derive(Debug)]
struct Record {
    distributor: Distributor<ExportEvent>,
    description: Description,
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
impl<T: Publisher> Eliminated<T> for Exporter {
    async fn handle(&mut self, id: IdOf<T>, _ctx: &mut Context<Self>) -> Result<(), Error> {
        log::info!("Publisher {} finished", id);
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
        if let Some(record) = self.recipients.get_mut(&path) {
            record.distributor.act_all(event).await?;
            Ok(())
        } else {
            Err(Reason::NoMetaForPath(path).into())
        }
    }
}

impl Exporter {
    fn declared_paths(&self) -> Vec<Description> {
        self.recipients
            .iter()
            .filter_map(|(_, record)| {
                if record.declared {
                    Some(record.description.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

#[async_trait]
impl ActionHandler<link::PathDeclared> for Exporter {
    async fn handle(
        &mut self,
        msg: link::PathDeclared,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let path = msg.description.path.clone();
        //let stream_type = msg.description.stream_type;
        log::info!("Declare path: {}", path);
        let entry = self.recipients.entry(path);
        match entry {
            Entry::Vacant(entry) => {
                let record = Record {
                    distributor: Distributor::new(),
                    description: msg.description.clone(),
                    declared: true,
                };
                entry.insert(record);
                let msg = PathNotification {
                    descriptions: vec![msg.description],
                };
                self.paths_trackers.act_all(msg).await?;
                Ok(())
            }
            Entry::Occupied(entry) => {
                let path = entry.get().description.path.clone();
                Err(Reason::AlreadyDeclaredPath(path).into())
            }
        }
    }
}

#[async_trait]
impl ActionHandler<link::SubscribeToPaths> for Exporter {
    async fn handle(
        &mut self,
        mut msg: link::SubscribeToPaths,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let descriptions = self.declared_paths();
        let event = PathNotification { descriptions };
        msg.recipient.act(event).await?;
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
        if let Some(record) = self.recipients.get_mut(&msg.path) {
            record.distributor.insert(msg.recipient);
            if record.distributor.len() == 1 && record.declared {
                self.provider()?.subscribe(path).await?;
            }
            Ok(())
        } else {
            Err(Reason::NotDeclaredPath(msg.path).into())
        }
    }
}

#[async_trait]
impl ActionHandler<link::UnsubscribeFromData> for Exporter {
    async fn handle(
        &mut self,
        msg: link::UnsubscribeFromData,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let path = msg.path.clone();
        if let Some(record) = self.recipients.get_mut(&msg.path) {
            record.distributor.remove(&msg.id);
            if record.distributor.is_empty() && record.declared {
                self.provider()?.unsubscribe(path).await?;
            }
            Ok(())
        } else {
            Err(Reason::NotDeclaredPath(msg.path).into())
        }
    }
}

#[async_trait]
impl<T: Publisher> ActionHandler<link::StartPublisher<T>> for Exporter {
    async fn handle(
        &mut self,
        msg: link::StartPublisher<T>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let publisher = T::create(msg.config, ctx.address().link(), &self.server);
        let address = ctx.spawn_actor(publisher, ());
        Ok(())
    }
}

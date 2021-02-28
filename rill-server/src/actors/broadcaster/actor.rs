use super::{link, PathNotification, Publisher};
use crate::actors::provider_session::ProviderSessionLink;
use crate::actors::server::RillServer;
use anyhow::Error;
use async_trait::async_trait;
use meio::{
    ActionHandler, Actor, Context, Distributor, Eliminated, IdOf, InterruptedBy, StartedBy,
};
use rill_protocol::provider::{Description, EntryId, Path};
use std::collections::{hash_map::Entry, HashMap};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Reason {
    #[error("No active provider available")]
    NoActiveSession,
    #[error("Path already declared {0}")]
    AlreadyDeclaredPath(Path),
}

#[derive(Debug)]
struct Record {
    description: Description,
    declared: bool,
}

/// The `Actor` that subscribes to data according to available `Path`s.
pub struct Broadcaster {
    provider: Option<(EntryId, ProviderSessionLink)>,
    paths_trackers: Distributor<PathNotification>,
    recipients: HashMap<Path, Record>,
}

impl Broadcaster {
    pub fn new() -> Self {
        Self {
            provider: None,
            paths_trackers: Distributor::new(),
            recipients: HashMap::new(),
        }
    }

    fn provider(&mut self) -> Result<&mut ProviderSessionLink, Reason> {
        if let Some((_, ref mut link)) = self.provider {
            Ok(link)
        } else {
            Err(Reason::NoActiveSession)
        }
    }

    async fn graceful_shutdown(&mut self, ctx: &mut Context<Self>) {
        if let Ok(provider) = self.provider() {
            provider.unsubscribe_all().await.ok();
        }
        ctx.shutdown();
    }
}

impl Actor for Broadcaster {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<RillServer> for Broadcaster {
    async fn handle(&mut self, _ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<RillServer> for Broadcaster {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.graceful_shutdown(ctx).await;
        Ok(())
    }
}

#[async_trait]
impl<T: Publisher> Eliminated<T> for Broadcaster {
    async fn handle(&mut self, id: IdOf<T>, _ctx: &mut Context<Self>) -> Result<(), Error> {
        log::info!("Publisher {} finished", id);
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::SessionLifetime> for Broadcaster {
    async fn handle(
        &mut self,
        msg: link::SessionLifetime,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        use link::SessionLifetime::*;
        match msg {
            Attached { name, session } => {
                let msg = PathNotification::Name { name: name.clone() };
                // Don't subscribe here till the stream (path) will be declared.
                self.provider = Some((name, session));
                self.paths_trackers.act_all(msg).await?;
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

/*
#[async_trait]
impl ActionHandler<link::DataReceived> for Broadcaster {
    async fn handle(
        &mut self,
        msg: link::DataReceived,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let path = msg.path.clone();
        let event = BroadcastEvent::BroadcastData {
            path: msg.path,
            event: msg.event,
        };
        if let Some(record) = self.recipients.get_mut(&path) {
            record.distributor.act_all(event).await?;
            Ok(())
        } else {
            Err(Reason::NoMetaForPath(path).into())
        }
    }
}
*/

impl Broadcaster {
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
impl ActionHandler<link::PathDeclared> for Broadcaster {
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
                    description: msg.description.clone(),
                    declared: true,
                };
                entry.insert(record);
                let msg = PathNotification::Paths {
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
impl ActionHandler<link::SubscribeToPaths> for Broadcaster {
    async fn handle(
        &mut self,
        mut msg: link::SubscribeToPaths,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.not_terminating()?;

        if let Some((ref name, _)) = self.provider {
            let event = PathNotification::Name { name: name.clone() };
            msg.recipient.act(event).await?;
        }

        // TODO: If there is no `Provider` do I have to ignore this broadcasting?
        let descriptions = self.declared_paths();
        let event = PathNotification::Paths { descriptions };
        msg.recipient.act(event).await?;
        self.paths_trackers.insert(msg.recipient);
        Ok(())
    }
}

/*
#[async_trait]
impl ActionHandler<link::SubscribeToData> for Broadcaster {
    async fn handle(
        &mut self,
        msg: link::SubscribeToData,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.not_terminating()?;
        let path = msg.path.clone();
        if let Some(record) = self.recipients.get_mut(&msg.path) {
            record.distributor.insert(msg.recipient);
            if record.distributor.len() == 1 && record.declared {
                log::info!("Subscribing to: {}", path);
                self.provider()?.subscribe(path).await?;
            }
            Ok(())
        } else {
            Err(Reason::NotDeclaredPath(msg.path).into())
        }
    }
}

#[async_trait]
impl ActionHandler<link::UnsubscribeFromData> for Broadcaster {
    async fn handle(
        &mut self,
        msg: link::UnsubscribeFromData,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let path = msg.path.clone();
        if let Some(record) = self.recipients.get_mut(&msg.path) {
            record.distributor.remove(&msg.id);
            if record.distributor.is_empty() && record.declared {
                log::info!("Unsubscribing from: {}", path);
                self.provider()?.unsubscribe(path).await?;
            }
            Ok(())
        } else {
            Err(Reason::NotDeclaredPath(msg.path).into())
        }
    }
}

#[async_trait]
impl<T: Publisher> ActionHandler<link::StartPublisher<T>> for Broadcaster {
    async fn handle(
        &mut self,
        msg: link::StartPublisher<T>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let publisher = T::create(msg.config, ctx.address().link(), &self.server);
        let _address = ctx.spawn_actor(publisher, ());
        Ok(())
    }
}
*/

use super::{link, PathNotification};
use anyhow::Error;
use async_trait::async_trait;
use meio::{ActionHandler, Actor, Context, Distributor, InterruptedBy, StartedBy};
use rill_protocol::provider::{Description, EntryId, Path};
use std::collections::{hash_map::Entry, HashMap};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Reason {
    #[error("Path already declared {0}")]
    AlreadyDeclaredPath(Path),
}

#[derive(Debug)]
struct Record {
    description: Description,
    declared: bool,
}

/// The `Actor` that informs about appeared providers and paths.
pub struct Broadcaster {
    name: Option<EntryId>,
    paths_trackers: Distributor<PathNotification>,
    recipients: HashMap<Path, Record>,
}

impl Broadcaster {
    pub fn new() -> Self {
        Self {
            name: None,
            paths_trackers: Distributor::new(),
            recipients: HashMap::new(),
        }
    }

    async fn graceful_shutdown(&mut self, ctx: &mut Context<Self>) {
        ctx.shutdown();
    }
}

impl Actor for Broadcaster {
    type GroupBy = ();
}

#[async_trait]
impl<T: Actor> StartedBy<T> for Broadcaster {
    async fn handle(&mut self, _ctx: &mut Context<Self>) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl<T: Actor> InterruptedBy<T> for Broadcaster {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.graceful_shutdown(ctx).await;
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
            Attached { name, .. } => {
                let msg = PathNotification::Name { name: name.clone() };
                // Don't subscribe here till the stream (path) will be declared.
                self.name = Some(name);
                self.paths_trackers.act_all(msg).await?;
            }
            Detached => {
                self.name.take();
                for record in self.recipients.values_mut() {
                    record.declared = false;
                }
            }
        }
        Ok(())
    }
}

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
impl ActionHandler<link::SubscribeToStructChanges> for Broadcaster {
    async fn handle(
        &mut self,
        mut msg: link::SubscribeToStructChanges,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.not_terminating()?;

        if let Some(name) = self.name.clone() {
            let event = PathNotification::Name { name };
            msg.recipient.act(event).await?;
        }

        let descriptions = self.declared_paths();
        let event = PathNotification::Paths { descriptions };
        msg.recipient.act(event).await?;
        self.paths_trackers.insert(msg.recipient);
        Ok(())
    }
}

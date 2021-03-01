use super::{Broadcaster, PathNotification};
use anyhow::Error;
use derive_more::From;
use meio::{Action, ActionHandler, ActionRecipient, Actor, Address};
use rill_protocol::provider::{Description, EntryId, Path};
use std::collections::HashSet;

/// This `Link` used by `Session` actor.
#[derive(Debug)]
pub struct BroadcasterLinkForClient {
    address: Address<Broadcaster>,
    active_streams: HashSet<Path>,
}

impl From<Address<Broadcaster>> for BroadcasterLinkForClient {
    fn from(address: Address<Broadcaster>) -> Self {
        Self {
            address,
            active_streams: HashSet::new(),
        }
    }
}

pub(super) struct SubscribeToStructChanges {
    pub recipient: Box<dyn ActionRecipient<PathNotification>>,
}

impl Action for SubscribeToStructChanges {}

impl BroadcasterLinkForClient {
    pub async fn subscribe_to_struct_changes<A>(&mut self, address: Address<A>) -> Result<(), Error>
    where
        A: Actor + ActionHandler<PathNotification>,
    {
        let recipient = Box::new(address);
        let msg = SubscribeToStructChanges { recipient };
        self.address.act(msg).await
    }
}

/// This `Link` used by `Session` actor.
#[derive(Debug, Clone, From)]
pub struct BroadcasterLinkForProvider {
    address: Address<Broadcaster>,
}

pub(super) enum SessionLifetime {
    Attached { name: EntryId },
    Detached,
}

impl Action for SessionLifetime {}

impl BroadcasterLinkForProvider {
    pub async fn session_attached(&mut self, name: EntryId) -> Result<(), Error> {
        let msg = SessionLifetime::Attached { name };
        self.address.act(msg).await
    }

    pub async fn session_detached(&mut self) -> Result<(), Error> {
        let msg = SessionLifetime::Detached;
        self.address.act(msg).await
    }
}

pub(super) struct PathDeclared {
    pub description: Description,
}

impl Action for PathDeclared {}

impl BroadcasterLinkForProvider {
    pub async fn path_declared(&mut self, description: Description) -> Result<(), Error> {
        let msg = PathDeclared { description };
        self.address.act(msg).await
    }
}

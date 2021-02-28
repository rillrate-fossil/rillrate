use super::ProviderSession;
use anyhow::Error;
use derive_more::From;
use meio::{Action, Address, Interaction};
use meio_connect::client::WsSender;
use rill_protocol::client::{ClientProtocol, ClientReqId, ClientResponse};
use rill_protocol::provider::{Path, ProviderReqId, ServerToProvider};
use rill_protocol::transport::WideEnvelope;
use std::collections::hash_map::{Entry, HashMap};
use thiserror::Error;

pub(super) type ClientSender = WsSender<WideEnvelope<ClientProtocol, ClientResponse>>;

#[derive(Debug, From)]
pub struct ProviderLink {
    address: Address<ProviderSession>,
}

impl ProviderLink {
    pub fn bind(&self, sender: ClientSender) -> BindedProviderLink {
        BindedProviderLink {
            sender,
            address: self.address.clone(),
            subscriptions: HashMap::new(),
        }
    }
}

#[derive(Debug, From)]
pub struct BindedProviderLink {
    sender: ClientSender,
    address: Address<ProviderSession>,
    subscriptions: HashMap<Path, ProviderReqId>,
}

pub(super) struct SubscribeToPath {
    pub path: Path,
    pub direct_id: ClientReqId,
    pub sender: ClientSender,
}

impl Interaction for SubscribeToPath {
    type Output = ProviderReqId;
}

impl BindedProviderLink {
    pub async fn subscribe(&mut self, path: Path, direct_id: ClientReqId) -> Result<(), Error> {
        match self.subscriptions.entry(path.clone()) {
            Entry::Vacant(entry) => {
                let sender = self.sender.clone();
                let msg = SubscribeToPath {
                    path,
                    direct_id,
                    sender,
                };
                let direct_id = self.address.interact_and_wait(msg).await?;
                entry.insert(direct_id);
                Ok(())
            }
            Entry::Occupied(_entry) => Err(Reason::AlreadySubscribed(path).into()),
        }
    }
}

#[derive(Debug, Error)]
enum Reason {
    #[error("Already subscribed {0}")]
    AlreadySubscribed(Path),
    #[error("Never subscribed {0}")]
    NeverSubscribed(Path),
}

/// It's not cloneable, because it tracks subscriptions.
#[derive(Debug)]
pub struct ProviderSessionLink {
    address: Address<ProviderSession>,
    subscriptions: HashMap<Path, ProviderReqId>,
}

impl From<Address<ProviderSession>> for ProviderSessionLink {
    fn from(address: Address<ProviderSession>) -> Self {
        Self {
            address,
            subscriptions: HashMap::new(),
        }
    }
}

pub(super) struct NewRequest {
    pub request: ServerToProvider,
}

impl Interaction for NewRequest {
    type Output = ProviderReqId;
}

impl ProviderSessionLink {
    pub async fn subscribe(&mut self, path: Path) -> Result<(), Error> {
        match self.subscriptions.entry(path.clone()) {
            Entry::Vacant(entry) => {
                let request = ServerToProvider::ControlStream { active: true, path };
                let msg = NewRequest { request };
                let direct_id = self.address.interact_and_wait(msg).await?;
                entry.insert(direct_id);
                Ok(())
            }
            Entry::Occupied(_entry) => Err(Reason::AlreadySubscribed(path).into()),
        }
    }
}

pub(super) struct SubRequest {
    pub direct_id: ProviderReqId,
    pub request: ServerToProvider,
}

impl Action for SubRequest {}

impl ProviderSessionLink {
    // TODO: Move to the separate link
    // TODO: Add id of the stream (returned before by subscribe call)
    pub async fn unsubscribe(&mut self, path: Path) -> Result<(), Error> {
        if let Some(direct_id) = self.subscriptions.remove(&path) {
            let request = ServerToProvider::ControlStream {
                active: false,
                path,
            };
            let msg = SubRequest { direct_id, request };
            self.address.act(msg).await
        } else {
            Err(Reason::NeverSubscribed(path).into())
        }
    }

    pub async fn unsubscribe_all(&mut self) -> Result<(), Error> {
        let paths: Vec<_> = self.subscriptions.keys().cloned().collect();
        for path in paths {
            self.unsubscribe(path).await?;
        }
        Ok(())
    }
}

use super::ProviderSession;
use anyhow::Error;
use meio::{Action, Address, Interaction};
use rill_protocol::provider::{Path, ProviderReqId, RillToProvider};
use std::collections::hash_map::{Entry, HashMap};
use thiserror::Error;

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
    pub request: RillToProvider,
}

impl Interaction for NewRequest {
    type Output = ProviderReqId;
}

impl ProviderSessionLink {
    pub async fn subscribe(&mut self, path: Path) -> Result<(), Error> {
        match self.subscriptions.entry(path.clone()) {
            Entry::Vacant(entry) => {
                let request = RillToProvider::ControlStream { active: true, path };
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
    pub request: RillToProvider,
}

impl Action for SubRequest {}

impl ProviderSessionLink {
    // TODO: Move to the separate link
    // TODO: Add id of the stream (returned before by subscribe call)
    pub async fn unsubscribe(&mut self, path: Path) -> Result<(), Error> {
        if let Some(direct_id) = self.subscriptions.remove(&path) {
            let request = RillToProvider::ControlStream {
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

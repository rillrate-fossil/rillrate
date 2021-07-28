use super::ProviderSession;
use crate::actors::client_session::ClientSender;
use anyhow::Error;
use derive_more::From;
use meio::{Action, Address, Interaction, InteractionTask};
use rill_protocol::io::client::ClientReqId;
use rill_protocol::io::provider::{Path, ProviderReqId, RecorderAction};

#[derive(Debug, From, Clone)]
pub struct ProviderLink {
    address: Address<ProviderSession>,
}

pub struct SubscribeToPath {
    pub path: Path,
    pub direct_id: ClientReqId,
    pub sender: ClientSender,
}

impl Interaction for SubscribeToPath {
    type Output = SubscriptionLink;
}

impl ProviderLink {
    pub fn subscribe(
        &mut self,
        path: Path,
        direct_id: ClientReqId,
        sender: ClientSender,
    ) -> InteractionTask<SubscribeToPath> {
        let msg = SubscribeToPath {
            path,
            direct_id,
            sender,
        };
        self.address.interact(msg)
    }
}

pub struct ActionOnPath {
    pub path: Path,
    pub direct_id: ClientReqId,
    pub sender: ClientSender,
    pub action: RecorderAction,
}

impl Action for ActionOnPath {}

impl ProviderLink {
    pub async fn action_on_path(
        &mut self,
        path: Path,
        direct_id: ClientReqId,
        sender: ClientSender,
        action: RecorderAction,
    ) -> Result<(), Error> {
        let msg = ActionOnPath {
            path: path.clone(),
            direct_id,
            sender,
            action,
        };
        self.address.act(msg).await
    }
}

#[derive(Debug)]
pub struct SubscriptionLink {
    pub(super) address: Address<ProviderSession>,
    pub(super) path: Path,
    pub(super) req_id: ProviderReqId,
}

pub struct UnsubscribeFromPath {
    pub path: Path,
    pub req_id: ProviderReqId,
}

impl Interaction for UnsubscribeFromPath {
    type Output = ();
}

impl SubscriptionLink {
    pub fn unsubscribe(self) -> InteractionTask<UnsubscribeFromPath> {
        let msg = UnsubscribeFromPath {
            path: self.path,
            req_id: self.req_id,
        };
        self.address.interact(msg)
    }
}

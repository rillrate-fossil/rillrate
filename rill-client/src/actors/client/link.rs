use super::RillClient;
use anyhow::Error;
use async_trait::async_trait;
use derive_more::From;
use futures::channel::mpsc;
use meio::{Action, Address, Interaction, InteractionTask};
use rill_protocol::client::ClientReqId;
use rill_protocol::provider::{Path, RillEvent};

#[derive(Debug, From)]
pub struct ClientLink {
    address: Address<RillClient>,
}

pub struct SubscribeToPath {
    pub path: Path,
    pub sender: mpsc::Sender<Vec<RillEvent>>,
}

impl Interaction for SubscribeToPath {
    type Output = ClientReqId;
}

impl ClientLink {
    pub fn subscribe_to_path(
        &mut self,
        path: Path,
        sender: mpsc::Sender<Vec<RillEvent>>,
    ) -> InteractionTask<SubscribeToPath> {
        let msg = SubscribeToPath { path, sender };
        self.address.interact(msg)
    }
}

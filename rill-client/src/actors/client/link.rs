use super::RillClient;
use anyhow::Error;
use async_trait::async_trait;
use derive_more::From;
use meio::{Action, Address, Interaction, InteractionTask};
use rill_protocol::client::ClientReqId;
use rill_protocol::provider::Path;

#[derive(Debug, From)]
pub struct ClientLink {
    address: Address<RillClient>,
}

pub struct SubscribeToPath {
    pub path: Path,
}

impl Interaction for SubscribeToPath {
    type Output = ClientReqId;
}

impl ClientLink {
    pub fn subscribe_to_path(&mut self, path: Path) -> InteractionTask<SubscribeToPath> {
        let msg = SubscribeToPath { path };
        self.address.interact(msg)
    }
}

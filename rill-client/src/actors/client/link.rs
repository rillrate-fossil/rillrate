use super::RillClient;
use anyhow::Error;
use async_trait::async_trait;
use derive_more::From;
use meio::{Action, Address, Interaction};
use rill_protocol::client::ClientReqId;
use rill_protocol::provider::Path;

#[derive(Debug, From)]
pub struct ClientLink {
    address: Address<RillClient>,
}

pub(super) struct SubscribeToPath {
    pub path: Path,
}

impl Interaction for SubscribeToPath {
    type Output = ClientReqId;
}

impl ClientLink {
    pub async fn subscribe_to_path(&mut self, path: Path) -> Result<ClientReqId, Error> {
        let msg = SubscribeToPath { path };
        self.address.interact_and_wait(msg).await
    }
}

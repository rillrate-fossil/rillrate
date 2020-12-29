use super::ProviderSession;
use anyhow::Error;
use derive_more::From;
use meio::prelude::{Action, Address};
use rill::protocol::{Path, RillToProvider};

#[derive(Debug, Clone, From)]
pub struct ProviderSessionLink {
    address: Address<ProviderSession>,
}

pub(super) struct ForwardRequest {
    pub request: RillToProvider,
}

impl Action for ForwardRequest {}

impl ProviderSessionLink {
    pub async fn subscribe(&mut self, path: Path) -> Result<(), Error> {
        let request = RillToProvider::ControlStream { active: true, path };
        let msg = ForwardRequest { request };
        self.address.act(msg).await
    }
}

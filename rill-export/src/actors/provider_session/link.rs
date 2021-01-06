use super::ProviderSession;
use anyhow::Error;
use derive_more::From;
use meio::prelude::{Action, Address, Interaction};
use rill_protocol::provider::{Path, ProviderReqId, RillToProvider};

/// It's not cloneable, because it tracks subscriptions.
#[derive(Debug, From)]
pub struct ProviderSessionLink {
    address: Address<ProviderSession>,
}

// TODO: Rename to NewRequest
pub(super) struct ForwardRequest {
    pub request: RillToProvider,
}

impl Interaction for ForwardRequest {
    type Output = ProviderReqId;
}

impl ProviderSessionLink {
    pub async fn subscribe(&mut self, path: Path) -> Result<(), Error> {
        let request = RillToProvider::ControlStream { active: true, path };
        let msg = ForwardRequest { request };
        self.address.interact(msg).await?;
        Ok(())
    }

    // TODO: Move to the separate link
    // TODO: Add id of the stream (returned before by subscribe call)
    pub async fn unsubscribe(&mut self, path: Path) -> Result<(), Error> {
        let request = RillToProvider::ControlStream {
            active: false,
            path,
        };
        let msg = ForwardRequest { request };
        self.address.interact(msg).await?;
        Ok(())
    }
}

use super::Exporter;
use anyhow::Error;
use derive_more::From;
use meio::prelude::{Action, Address};

#[derive(Debug, Clone, From)]
pub struct ExporterLink {
    address: Address<Exporter>,
}

pub(super) enum SessionLifetime {
    Attached,
    Detached,
}

impl Action for SessionLifetime {}

impl ExporterLink {
    pub async fn session_attached(&mut self) -> Result<(), Error> {
        self.address.act(SessionLifetime::Attached).await
    }

    pub async fn session_detached(&mut self) -> Result<(), Error> {
        self.address.act(SessionLifetime::Detached).await
    }
}

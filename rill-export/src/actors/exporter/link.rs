use super::Exporter;
use crate::actors::session::SessionLink;
use anyhow::Error;
use derive_more::From;
use meio::prelude::{Action, Address};
use rill::protocol::{Description, Path, StreamType};

#[derive(Debug, Clone, From)]
pub struct ExporterLink {
    address: Address<Exporter>,
}

pub(super) enum SessionLifetime {
    Attached { session: SessionLink },
    Detached,
}

impl Action for SessionLifetime {}

impl ExporterLink {
    pub async fn session_attached(&mut self, session: SessionLink) -> Result<(), Error> {
        let msg = SessionLifetime::Attached { session };
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

impl ExporterLink {
    pub async fn path_declared(&mut self, description: Description) -> Result<(), Error> {
        let msg = PathDeclared { description };
        self.address.act(msg).await
    }
}

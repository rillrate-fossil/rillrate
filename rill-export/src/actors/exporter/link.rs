use super::Exporter;
use crate::actors::provider_session::ProviderSessionLink;
use anyhow::Error;
use derive_more::From;
use meio::prelude::{Action, Address};
use rill::protocol::{Description, Path, RillData};
use std::time::Duration;

#[derive(Debug, Clone, From)]
pub struct ExporterLinkForData {
    address: Address<Exporter>,
}

pub(super) enum SessionLifetime {
    Attached { session: ProviderSessionLink },
    Detached,
}

impl Action for SessionLifetime {}

impl ExporterLinkForData {
    pub async fn session_attached(&mut self, session: ProviderSessionLink) -> Result<(), Error> {
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

impl ExporterLinkForData {
    pub async fn path_declared(&mut self, description: Description) -> Result<(), Error> {
        let msg = PathDeclared { description };
        self.address.act(msg).await
    }
}

pub(super) struct DataReceived {
    pub path: Path,
    pub timestamp: Duration,
    pub data: RillData,
}

impl Action for DataReceived {}

impl ExporterLinkForData {
    pub async fn data_received(
        &mut self,
        path: Path,
        timestamp: Duration,
        data: RillData,
    ) -> Result<(), Error> {
        let msg = DataReceived {
            path,
            timestamp,
            data,
        };
        self.address.act(msg).await
    }
}

#[derive(Debug, Clone, From)]
pub struct ExporterLinkForCtrl {
    address: Address<Exporter>,
}

pub(super) struct ExportPath {
    pub path: Path,
}

impl Action for ExportPath {}

impl ExporterLinkForCtrl {
    // TODO: Use Pattern instead of Path
    pub async fn export_path(&mut self, path: Path) -> Result<(), Error> {
        let msg = ExportPath { path };
        self.address.act(msg).await
    }
}

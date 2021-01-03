use super::Exporter;
use crate::actors::provider_session::ProviderSessionLink;
use anyhow::Error;
use derive_more::From;
use meio::prelude::{Action, Address, Interaction};
use rill_protocol::provider::{Description, Path, RillData};
use std::collections::HashSet;
use std::time::Duration;

/// This `Link` used by `Session` actor.
#[derive(Debug, Clone, From)]
pub struct ExporterLinkForClient {
    address: Address<Exporter>,
}

pub(super) struct GetPaths;

impl Interaction for GetPaths {
    type Output = HashSet<Path>;
}

impl ExporterLinkForClient {
    pub async fn get_paths(&mut self) -> Result<HashSet<Path>, Error> {
        self.address.interact(GetPaths).await
    }
}

/// This `Link` used by `Session` actor.
#[derive(Debug, Clone, From)]
pub struct ExporterLinkForProvider {
    address: Address<Exporter>,
}

pub(super) enum SessionLifetime {
    Attached { session: ProviderSessionLink },
    Detached,
}

impl Action for SessionLifetime {}

impl ExporterLinkForProvider {
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

impl ExporterLinkForProvider {
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

impl ExporterLinkForProvider {
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

/// This `Link` used by `Tuner` actor.
#[derive(Debug, Clone, From)]
pub struct ExporterLinkForTuner {
    address: Address<Exporter>,
}

pub(super) struct ExportPath {
    pub path: Path,
}

impl Action for ExportPath {}

impl ExporterLinkForTuner {
    // TODO: Use Pattern instead of Path
    pub async fn export_path(&mut self, path: Path) -> Result<(), Error> {
        let msg = ExportPath { path };
        self.address.act(msg).await
    }
}

pub(super) struct StartPrometheus {}

impl Action for StartPrometheus {}

impl ExporterLinkForTuner {
    pub async fn start_prometheus(&mut self) -> Result<(), Error> {
        let msg = StartPrometheus {};
        self.address.act(msg).await
    }
}

pub(super) struct StartGraphite {}

impl Action for StartGraphite {}

impl ExporterLinkForTuner {
    pub async fn start_graphite(&mut self) -> Result<(), Error> {
        let msg = StartGraphite {};
        self.address.act(msg).await
    }
}

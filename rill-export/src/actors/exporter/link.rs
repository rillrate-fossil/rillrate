use super::{ExportEvent, Exporter, PathNotification};
use crate::actors::provider_session::ProviderSessionLink;
use crate::config::{GraphiteConfig, PrometheusConfig};
use anyhow::Error;
use derive_more::From;
use meio::prelude::{
    Action, ActionHandler, ActionRecipient, Actor, Address, Interaction, TryConsumer,
};
use rill_protocol::provider::{Description, Path, RillData};
use std::collections::HashSet;
use std::time::Duration;
use tokio::sync::broadcast;

/// This `Link` used by `Session` actor.
#[derive(Debug, Clone, From)]
pub struct ExporterLinkForClient {
    address: Address<Exporter>,
}

/*
pub(super) struct GetPaths;

impl Interaction for GetPaths {
    type Output = HashSet<Path>;
}

impl ExporterLinkForClient {
    pub async fn get_paths(&mut self) -> Result<HashSet<Path>, Error> {
        self.address.interact(GetPaths).await
    }
}

pub(super) struct GetProviderSession;

impl Interaction for GetProviderSession {
    type Output = ProviderSessionLink;
}

impl ExporterLinkForClient {
    pub async fn get_provider_session(&mut self) -> Result<ProviderSessionLink, Error> {
        self.address.interact(GetProviderSession).await
    }
}
*/

pub(super) struct SubscribeToData {
    pub path: Path,
    pub recipient: Box<dyn ActionRecipient<ExportEvent>>,
}

impl Action for SubscribeToData {}

impl ExporterLinkForClient {
    // TODO: Use Pattern instead of Path
    pub async fn subscribe_to_data<A>(
        &mut self,
        path: Path,
        address: Address<A>,
    ) -> Result<(), Error>
    where
        A: Actor + ActionHandler<ExportEvent>,
    {
        let recipient = Box::new(address);
        let msg = SubscribeToData { path, recipient };
        self.address.act(msg).await
    }
}

pub(super) struct SubscribeToPaths {
    pub recipient: Box<dyn ActionRecipient<PathNotification>>,
}

impl Action for SubscribeToPaths {}

impl ExporterLinkForClient {
    pub async fn subscribe_to_paths<A>(&mut self, address: Address<A>) -> Result<(), Error>
    where
        A: Actor + ActionHandler<PathNotification>,
    {
        let recipient = Box::new(address);
        let msg = SubscribeToPaths { recipient };
        self.address.act(msg).await
    }
}

pub(super) struct StartPrometheus {
    pub config: PrometheusConfig,
}

impl Action for StartPrometheus {}

impl ExporterLinkForClient {
    pub async fn start_prometheus(&mut self, config: PrometheusConfig) -> Result<(), Error> {
        let msg = StartPrometheus { config };
        self.address.act(msg).await
    }
}

pub(super) struct StartGraphite {
    pub config: GraphiteConfig,
}

impl Action for StartGraphite {}

impl ExporterLinkForClient {
    pub async fn start_graphite(&mut self, config: GraphiteConfig) -> Result<(), Error> {
        let msg = StartGraphite { config };
        self.address.act(msg).await
    }
}

/*
pub(super) struct GraspExportStream<A: Actor> {
    pub listener: Address<A>,
}

impl<A: Actor> Action for GraspExportStream<A> {}

impl ExporterLinkForClient {
    pub async fn grasp_export_stream<A>(&mut self, address: Address<A>) -> Result<(), Error>
    where
        A: Actor + TryConsumer<ExportEvent, Error = broadcast::RecvError>,
    {
        let msg = GraspExportStream { listener: address };
        self.address.act(msg).await
    }
}
*/

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

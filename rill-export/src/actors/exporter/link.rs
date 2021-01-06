use super::{ExportEvent, Exporter, PathNotification, Publisher};
use crate::actors::provider_session::ProviderSessionLink;
use anyhow::Error;
use derive_more::From;
use meio::prelude::{Action, ActionHandler, ActionRecipient, Actor, Address, Id};
use rill_protocol::provider::{Description, Path, RillData};
use std::time::Duration;

/// This `Link` used by `Session` actor.
#[derive(Debug, Clone, From)]
pub struct ExporterLinkForClient {
    address: Address<Exporter>,
}

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

pub(super) struct UnsubscribeFromData {
    pub path: Path,
    pub id: Id,
}

impl Action for UnsubscribeFromData {}

impl ExporterLinkForClient {
    pub async fn unsubscribe_from_data<A>(
        &mut self,
        path: Path,
        address: &Address<A>,
    ) -> Result<(), Error>
    where
        A: Actor + ActionHandler<ExportEvent>,
    {
        let id = address.id().into();
        let msg = UnsubscribeFromData { path, id };
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

pub(super) struct StartPublisher<T: Publisher> {
    pub config: T::Config,
}

impl<T: Publisher> Action for StartPublisher<T> {}

impl ExporterLinkForClient {
    pub async fn start_publisher<T>(&mut self, config: T::Config) -> Result<(), Error>
    where
        T: Publisher,
    {
        let msg: StartPublisher<T> = StartPublisher { config };
        self.address.act(msg).await
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

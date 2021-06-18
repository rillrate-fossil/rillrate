pub mod parcel;

use crate::actors::engine::RillEngine;
use crate::actors::recorder::{Recorder, RecorderLink};
use crate::config::EngineConfig;
use crate::tracers::meta::PathTracer;
use anyhow::Error;
use async_trait::async_trait;
use meio::{
    ActionHandler, Actor, Context, Eliminated, Id, IdOf, InstantActionHandler, InterruptedBy,
    StartedBy, TaskEliminated, TaskError,
};
use meio_connect::{
    client::{WsClient, WsClientStatus, WsSender},
    WsIncoming,
};
use rill_protocol::flow::core;
use rill_protocol::flow::meta::path::PATHS;
use rill_protocol::io::provider::{
    Description, ProviderProtocol, ProviderToServer, ServerToProvider,
};
use rill_protocol::io::transport::{Direction, Envelope, WideEnvelope};
use rill_protocol::pathfinder::{Pathfinder, Record};
use std::collections::HashMap;
use std::time::Duration;

/// Wrapper for WebSocket connection for sending responses (notifications) to a server.
#[derive(Default, Clone)]
pub(crate) struct RillSender {
    sender: Option<WsSender<WideEnvelope<ProviderProtocol, ProviderToServer>>>,
}

impl RillSender {
    /*
    pub fn is_connected(&self) -> bool {
        self.sender.is_some()
    }
    */

    fn set(&mut self, sender: WsSender<WideEnvelope<ProviderProtocol, ProviderToServer>>) {
        self.sender = Some(sender);
    }

    pub fn reset(&mut self) {
        self.sender.take();
    }

    pub fn response(&mut self, direction: Direction<ProviderProtocol>, data: ProviderToServer) {
        if let Some(sender) = self.sender.as_ref() {
            let envelope = WideEnvelope { direction, data };
            sender.send(envelope);
        } else {
            log::error!("Can't send a response. Not connected.");
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    WsConnection,
    /// Interactions
    ActiveRequests,
    ParcelStream,
    Recorders,
}

pub struct RillConnector {
    url: String,
    config: EngineConfig,
    sender: RillSender,
    recorders: Pathfinder<RecorderLink>,
    registered: HashMap<Id, Description>,
    path_flow: PathTracer,
}

impl RillConnector {
    pub fn new(config: EngineConfig) -> Self {
        let paths = PATHS.root();
        Self {
            url: config.node_url(),
            config,
            sender: RillSender::default(),
            recorders: Pathfinder::default(),
            registered: HashMap::new(),
            path_flow: PathTracer::new(paths),
        }
    }

    fn send_global(&mut self, msg: ProviderToServer) {
        self.sender.response(Direction::broadcast(), msg);
    }
}

#[async_trait]
impl Actor for RillConnector {
    type GroupBy = Group;

    fn name(&self) -> String {
        format!("RillConnector({})", &self.url)
    }
}

#[async_trait]
impl StartedBy<RillEngine> for RillConnector {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        // TODO: Replace with strum iter
        ctx.termination_sequence(vec![
            Group::ActiveRequests,
            Group::WsConnection,
            Group::ParcelStream,
            Group::Recorders,
        ]);

        self.attach_distributor(ctx).await?;

        let client = WsClient::new(
            self.config.node_url(),
            Some(Duration::from_secs(1)),
            ctx.address().clone(),
        );
        ctx.spawn_task(client, (), Group::WsConnection);

        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<RillEngine> for RillConnector {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.detach_distributor();
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl InstantActionHandler<WsClientStatus<ProviderProtocol>> for RillConnector {
    async fn handle(
        &mut self,
        status: WsClientStatus<ProviderProtocol>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        match status {
            WsClientStatus::Connected { sender } => {
                // TODO: Resend new sender to all `Recorders`
                self.sender.set(sender);

                for desc in self.registered.values_mut() {
                    // TODO: Use `Pathfinder::walk` to perform that
                    let path = &desc.path;
                    let link = self.recorders.find_mut(path).and_then(Record::get_link_mut);
                    if let Some(link) = link {
                        // TODO: Run in parallel for all links
                        link.connected(self.sender.clone()).await.ok();
                    }
                }

                let entry_id = self.config.provider_name();
                let provider_type = self.config.provider_type();
                let msg = ProviderToServer::Declare {
                    entry_id,
                    provider_type,
                };
                self.send_global(msg);
            }
            WsClientStatus::Failed { reason } => {
                log::error!("Connection failed: {}", reason);
                // TODO: Try to reconnect...

                // TODO: DRY!!! See above! It's the same (
                for desc in self.registered.values_mut() {
                    // TODO: Use `Pathfinder::walk` to perform that
                    let path = &desc.path;
                    let link = self.recorders.find_mut(path).and_then(Record::get_link_mut);
                    if let Some(link) = link {
                        // TODO: Run in parallel for all links
                        link.disconnected().await.ok();
                    }
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<WsIncoming<Envelope<ProviderProtocol, ServerToProvider>>> for RillConnector {
    async fn handle(
        &mut self,
        msg: WsIncoming<Envelope<ProviderProtocol, ServerToProvider>>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let envelope = msg.0;
        log::trace!("Incoming request: {:?}", envelope);
        let direct_id = envelope.direct_id;
        let path = envelope.data.path;
        let recorder_link = self
            .recorders
            .find_mut(&path)
            .and_then(Record::get_link_mut);
        if let Some(recorder) = recorder_link {
            let request = envelope.data.request;
            recorder.do_path_request(direct_id, request).await?;
        } else {
            log::warn!("Path not found: {:?}", path);
            let msg = ProviderToServer::Error {
                reason: format!("path {} not found", path),
            };
            self.sender.response(direct_id.into(), msg);
        }
        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<WsClient<ProviderProtocol, Self>, ()> for RillConnector {
    async fn handle(
        &mut self,
        _id: IdOf<WsClient<ProviderProtocol, Self>>,
        _tag: (),
        _result: Result<(), TaskError>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        // TODO: Drop unfinished tasks
        Ok(())
    }
}

#[async_trait]
impl<T: core::Flow> Eliminated<Recorder<T>> for RillConnector {
    async fn handle(
        &mut self,
        id: IdOf<Recorder<T>>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let id: Id = id.into();
        if let Some(desc) = self.registered.remove(&id) {
            let path = &desc.path;
            let link = self.recorders.find_mut(&path).and_then(Record::take_link);
            if link.is_some() {
                self.path_flow.del(path.to_owned());
            } else {
                log::error!("Recorder {:?} was registered without a link (lost).", id);
            }
        } else {
            log::error!("Recorder {:?} wasn't registered.", id);
        }
        Ok(())
    }
}

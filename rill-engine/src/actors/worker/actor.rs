use crate::actors::engine::RillEngine;
use crate::actors::recorder::{Recorder, RecorderLink};
use crate::config::EngineConfig;
use crate::state;
use crate::tracers::meta::PathTracer;
use anyhow::Error;
use async_trait::async_trait;
use meio::{
    ActionHandler, Actor, Consumer, Context, Eliminated, Id, IdOf, InstantActionHandler,
    InterruptedBy, Parcel, StartedBy, TaskEliminated, TaskError,
};
use meio_connect::{
    client::{WsClient, WsClientStatus, WsSender},
    WsIncoming,
};
use rill_protocol::flow::data;
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
    pub fn is_connected(&self) -> bool {
        self.sender.is_some()
    }

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
    UpgradeStream,
    Recorders,
}

pub struct RillWorker {
    url: String,
    config: EngineConfig,
    sender: RillSender,
    recorders: Pathfinder<RecorderLink>,
    describe: bool,
    registered: HashMap<Id, Description>,
    path_flow: PathTracer,
}

impl RillWorker {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            url: config.node_url(),
            config,
            sender: RillSender::default(),
            recorders: Pathfinder::default(),
            describe: false,
            registered: HashMap::new(),
            path_flow: PathTracer::new(),
        }
    }

    fn send_global(&mut self, msg: ProviderToServer) {
        self.sender.response(Direction::broadcast(), msg);
    }
}

#[async_trait]
impl Actor for RillWorker {
    type GroupBy = Group;

    fn name(&self) -> String {
        format!("RillWorker({})", &self.url)
    }
}

#[async_trait]
impl StartedBy<RillEngine> for RillWorker {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![
            Group::WsConnection,
            Group::UpgradeStream,
            Group::Recorders,
        ]);

        let rx = state::RILL_LINK
            .take_receiver()
            .await
            .ok_or_else(|| Error::msg("Receiver already taken"))?;
        ctx.attach(rx, (), Group::UpgradeStream);

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
impl InterruptedBy<RillEngine> for RillWorker {
    async fn handle(&mut self, _ctx: &mut Context<Self>) -> Result<(), Error> {
        // Closes the control channel and with then it will be finished
        state::RILL_LINK.sender.close_channel();
        Ok(())
    }
}

#[async_trait]
impl InstantActionHandler<WsClientStatus<ProviderProtocol>> for RillWorker {
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

                /*
                let entry_id = self.config.provider_name();
                let msg = ProviderToServer::Declare { entry_id };
                self.send_global(msg);
                */
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
impl ActionHandler<WsIncoming<Envelope<ProviderProtocol, ServerToProvider>>> for RillWorker {
    async fn handle(
        &mut self,
        msg: WsIncoming<Envelope<ProviderProtocol, ServerToProvider>>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let envelope = msg.0;
        log::trace!("Incoming request: {:?}", envelope);
        let direct_id = envelope.direct_id;
        match envelope.data {
            ServerToProvider::ControlStream { path, active } => {
                log::debug!("Switching the stream {:?} to {:?}", path, active);
                let recorder_link = self
                    .recorders
                    .find_mut(&path)
                    .and_then(Record::get_link_mut);
                if let Some(recorder) = recorder_link {
                    recorder.control_stream(direct_id, active).await?;
                } else {
                    log::warn!("Path not found: {:?}", path);
                    let msg = ProviderToServer::Error {
                        reason: format!("path {} not found", path),
                    };
                    self.sender.response(direct_id.into(), msg);
                }
            }
            /*
            ServerToProvider::Describe { active } => {
                // TODO: Check or use `Direction` here?
                let dont_send_empty = !self.registered.is_empty();
                let not_described_yet = !self.describe;
                if active && not_described_yet && dont_send_empty {
                    // Send all exist paths
                    let list = self
                        .registered
                        .values()
                        .map(|desc| Description::clone(desc))
                        .collect();
                    let msg = ProviderToServer::Description { list };
                    self.send_global(msg);
                }
                self.describe = active;
            }
            */
            req => {
                log::error!("TODO: Request {:?} is not implemented yet.", req);
            }
        }
        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<WsClient<ProviderProtocol, Self>, ()> for RillWorker {
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
impl<T: data::Flow> InstantActionHandler<state::RegisterTracer<T>> for RillWorker {
    async fn handle(
        &mut self,
        msg: state::RegisterTracer<T>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let description = msg.description;
        let path = description.path.clone();
        log::info!("Add tracer: {:?}", path);
        let record = self.recorders.dig(path.clone());
        if record.get_link().is_none() {
            let packed_desc = description.to_description()?;
            let sender = self.sender.clone();
            //let link = ctx.address().link();
            let actor = Recorder::new(description, sender, msg.mode);
            let recorder = ctx.spawn_actor(actor, Group::Recorders);
            record.set_link(recorder.link());
            // Send a description that's new tracer added
            self.registered
                .insert(recorder.id().into(), packed_desc.clone());
            self.path_flow.add(path, packed_desc);
            // TODO: Remove that notification below
            /*
            if self.sender.is_connected() {
                let msg = ProviderToServer::Description {
                    list: vec![packed_desc],
                };
                self.send_global(msg);
            }
            */
        } else {
            log::error!("Provider for {} already registered.", path);
        }
        Ok(())
    }
}

#[async_trait]
impl<T: data::Flow> Eliminated<Recorder<T>> for RillWorker {
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

#[async_trait]
impl Consumer<Parcel<Self>> for RillWorker {
    async fn handle(&mut self, parcel: Parcel<Self>, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.address().unpack_parcel(parcel)
    }

    async fn finished(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

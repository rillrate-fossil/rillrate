use crate::actors::engine::RillEngine;
use crate::actors::recorder::{Recorder, RecorderLink};
use crate::config::ProviderConfig;
use crate::state;
use crate::tracers::tracer::TracerEvent;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    ActionHandler, Actor, Consumer, Context, Eliminated, Id, IdOf, InstantActionHandler,
    InterruptedBy, Parcel, StartedBy, TaskEliminated, TaskError,
};
use meio_connect::{
    client::{WsClient, WsClientStatus, WsSender},
    WsIncoming,
};
use rill_protocol::pathfinder::{Pathfinder, Record};
use rill_protocol::provider::{
    Description, Direction, Envelope, RillProtocol, RillToProvider, RillToServer, WideEnvelope,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Wrapper for WebSocket connection for sending responses (notifications) to a server.
#[derive(Default, Clone)]
pub(crate) struct RillSender {
    sender: Option<WsSender<WideEnvelope<RillProtocol, RillToServer>>>,
}

impl RillSender {
    fn set(&mut self, sender: WsSender<WideEnvelope<RillProtocol, RillToServer>>) {
        self.sender = Some(sender);
    }

    pub fn reset(&mut self) {
        self.sender.take();
    }

    pub fn response(&mut self, direction: Direction<RillProtocol>, data: RillToServer) {
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
    config: ProviderConfig,
    sender: RillSender,
    recorders: Pathfinder<RecorderLink>,
    describe: bool,
    registered: HashMap<Id, Arc<Description>>,
}

impl RillWorker {
    pub fn new(config: Option<ProviderConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            sender: RillSender::default(),
            recorders: Pathfinder::default(),
            describe: false,
            registered: HashMap::new(),
        }
    }

    fn send_global(&mut self, msg: RillToServer) {
        self.sender.response(Direction::broadcast(), msg);
    }
}

#[async_trait]
impl Actor for RillWorker {
    type GroupBy = Group;

    /*
    fn name(&self) -> String {
        format!("RillWorker({})", self.config.url())
    }
    */
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
        ctx.attach(rx, Group::UpgradeStream);

        let client = WsClient::new(
            self.config.node_url(),
            Some(Duration::from_secs(1)),
            ctx.address().clone(),
        );
        ctx.spawn_task(client, Group::WsConnection);

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
impl InstantActionHandler<WsClientStatus<RillProtocol>> for RillWorker {
    async fn handle(
        &mut self,
        status: WsClientStatus<RillProtocol>,
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
                let msg = RillToServer::Declare { entry_id };
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
impl ActionHandler<WsIncoming<Envelope<RillProtocol, RillToProvider>>> for RillWorker {
    async fn handle(
        &mut self,
        msg: WsIncoming<Envelope<RillProtocol, RillToProvider>>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let envelope = msg.0;
        log::trace!("Incoming request: {:?}", envelope);
        let direct_id = envelope.direct_id;
        match envelope.data {
            RillToProvider::ControlStream { path, active } => {
                log::debug!("Switching the stream {:?} to {:?}", path, active);
                let recorder_link = self
                    .recorders
                    .find_mut(&path)
                    .and_then(Record::get_link_mut);
                if let Some(recorder) = recorder_link {
                    recorder.control_stream(direct_id, active).await?;
                } else {
                    log::warn!("Path not found: {:?}", path);
                    let msg = RillToServer::Error {
                        reason: format!("path {} not found", path),
                    };
                    self.sender.response(direct_id.into(), msg);
                }
            }
            RillToProvider::Describe { active } => {
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
                    let msg = RillToServer::Description { list };
                    self.send_global(msg);
                }
                self.describe = active;
            }
            req => {
                log::error!("TODO: Request {:?} is not implemented yet.", req);
            }
        }
        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<WsClient<RillProtocol, Self>> for RillWorker {
    async fn handle(
        &mut self,
        _id: IdOf<WsClient<RillProtocol, Self>>,
        _result: Result<(), TaskError>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        // TODO: Drop unfinished tasks
        Ok(())
    }
}

#[async_trait]
impl<T: TracerEvent> InstantActionHandler<state::RegisterTracer<T>> for RillWorker {
    async fn handle(
        &mut self,
        msg: state::RegisterTracer<T>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let description = msg.description;
        let receiver = msg.receiver;
        let path = description.path.clone();
        log::info!("Add tracer: {:?}", path);
        let record = self.recorders.dig(path.clone());
        if record.get_link().is_none() {
            let sender = self.sender.clone();
            //let link = ctx.address().link();
            let actor = Recorder::new(description.clone(), sender, receiver);
            let recorder = ctx.spawn_actor(actor, Group::Recorders);
            record.set_link(recorder.link());
            self.registered.insert(recorder.id().into(), description);
        } else {
            log::error!("Provider for {} already registered.", path);
        }
        Ok(())
    }
}

#[async_trait]
impl<T: TracerEvent> Eliminated<Recorder<T>> for RillWorker {
    async fn handle(
        &mut self,
        id: IdOf<Recorder<T>>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let id: Id = id.into();
        if let Some(desc) = self.registered.remove(&id) {
            let path = &desc.path;
            let link = self.recorders.find_mut(&path).and_then(Record::take_link);
            if link.is_none() {
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
    fn stream_group(&self) -> Self::GroupBy {
        Group::UpgradeStream
    }

    async fn handle(
        &mut self,
        chunk: Vec<Parcel<Self>>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        for parcel in chunk {
            if let Err(err) = ctx.address().unpack_parcel(parcel) {
                log::error!("Can't unpack a parcel for the worker: {}", err);
            }
        }
        Ok(())
    }

    async fn finished(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}
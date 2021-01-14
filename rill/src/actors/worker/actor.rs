use crate::actors::supervisor::RillSupervisor;
use crate::providers::provider::DataEnvelope;
use crate::state::RegisterProvider;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    ActionHandler, Actor, Consumer, Context, IdOf, InstantActionHandler, InterruptedBy, StartedBy,
    TaskEliminated, TaskError,
};
use meio_connect::{
    client::{WsClient, WsClientStatus, WsSender},
    WsIncoming,
};
use rill_protocol::pathfinder::{Pathfinder, Record};
use rill_protocol::provider::{
    Description, Direction, EntryId, EntryType, Envelope, Path, ProviderReqId, RillProtocol,
    RillToProvider, RillToServer, WideEnvelope,
};
use slab::Slab;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::watch;

// TODO: Add `DirectionSet` that can give `Direction` value that depends
// of the 0,1,N items contained

struct Joint {
    // TODO: How about remove it? Looks like it's enough to have `subscribers` field.
    idx: usize,
    description: Arc<Description>,
    activator: watch::Sender<Option<usize>>,
    /// Remote Subscribers on the server.
    subscribers: HashSet<ProviderReqId>,
}

impl Joint {
    fn new(
        idx: usize,
        description: Arc<Description>,
        activator: watch::Sender<Option<usize>>,
    ) -> Self {
        Self {
            idx,
            description,
            activator,
            subscribers: HashSet::new(),
        }
    }

    /// It's force to show that's just changes the flag without any checks
    /// the data required or not.
    fn force_switch(&mut self, active: bool) {
        let flag = if active { Some(self.idx) } else { None };
        // TODO: Implement Provider unregistering
        // TODO: Check the watch is not closed
        if let Err(err) = self.activator.send(flag) {
            log::error!(
                "Can't switch the stream {} to {}: {}",
                self.description.path,
                active,
                err
            );
        }
    }

    fn try_switch_on(&mut self) {
        if !self.subscribers.is_empty() {
            self.force_switch(true);
        }
    }

    fn try_switch_off(&mut self) {
        if self.subscribers.is_empty() {
            self.force_switch(false);
        }
    }
}

/// Wrapper for WebSocket connection for sending responses (notifications) to a server.
#[derive(Default)]
struct RillSender {
    sender: Option<WsSender<WideEnvelope<RillProtocol, RillToServer>>>,
}

impl RillSender {
    fn set(&mut self, sender: WsSender<WideEnvelope<RillProtocol, RillToServer>>) {
        self.sender = Some(sender);
    }

    fn response(&mut self, direction: Direction<RillProtocol>, data: RillToServer) {
        if let Some(sender) = self.sender.as_mut() {
            let envelope = WideEnvelope { direction, data };
            sender.send(envelope);
        } else {
            log::error!("Can't send a response. Not connected.");
        }
    }
}

pub struct RillWorker {
    url: String,
    entry_id: EntryId,
    /// Active WebScoket outgoing connection
    sender: RillSender,
    index: Pathfinder<usize>,
    // TODO: Use TypedSlab here?
    joints: Slab<Joint>,
    describe: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    // TODO: Use it for coroutine-based streams (maybe)
    Subscriptions,
    WsConnection,
}

#[async_trait]
impl Actor for RillWorker {
    type GroupBy = Group;

    fn name(&self) -> String {
        format!("RillWorker({})", self.url)
    }
}

impl RillWorker {
    pub fn new(entry_id: EntryId) -> Self {
        let link = format!("ws://127.0.0.1:{}/live/provider", rill_protocol::PORT.get());
        Self {
            url: link,
            entry_id,
            sender: RillSender::default(),
            index: Pathfinder::default(),
            joints: Slab::new(),
            describe: false,
        }
    }

    fn send_global(&mut self, msg: RillToServer) {
        self.sender.response(Direction::broadcast(), msg);
    }

    fn send_list_for(&mut self, direct_id: ProviderReqId, path: &Path) {
        let msg;
        if let Some(records) = self.index.find(path).map(Record::list) {
            let entries = records
                .map(|(entry_id, idx)| {
                    let stream_type = idx
                        .and_then(|idx| {
                            self.joints.get(*idx).map(|joint| {
                                let stream_type = joint.description.stream_type;
                                EntryType::Stream(stream_type)
                            })
                        })
                        .unwrap_or(EntryType::Container);
                    (entry_id, stream_type)
                })
                .collect();
            log::trace!("Entries list for {:?}: {:?}", path, entries);
            msg = RillToServer::Entries { entries };
        } else {
            log::trace!("No entry for {:?} to get a list", path);
            let reason = "entry not found".to_string();
            msg = RillToServer::Error { reason };
        }
        self.sender.response(direct_id.into(), msg);
    }

    fn stop_all(&mut self) {
        for (_, joint) in self.joints.iter_mut() {
            // TODO: Check there is no alive sessions or remove them before checking
            joint.try_switch_off();
        }
    }
}

#[async_trait]
impl StartedBy<RillSupervisor> for RillWorker {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![Group::Subscriptions, Group::WsConnection]);
        let client = WsClient::new(
            self.url.clone(),
            Some(Duration::from_secs(1)),
            ctx.address().clone(),
        );
        ctx.spawn_task(client, Group::WsConnection);
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<RillSupervisor> for RillWorker {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        // TODO: Stop all streams and send errors to subscribers!
        ctx.shutdown();
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
impl Consumer<RegisterProvider> for RillWorker {
    async fn handle(
        &mut self,
        event: RegisterProvider,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let RegisterProvider {
            description,
            mode,
            rx,
        } = event;
        let activator = mode.activator;
        let path = description.path.clone();
        log::info!("Add provider: {:?}", path);
        let record = self.index.dig(path.clone());
        if record.get_link().is_none() {
            let entry = self.joints.vacant_entry();
            let idx = entry.key();
            let joint = Joint::new(idx, description, activator);
            let joint_ref = entry.insert(joint);
            ctx.address().attach(rx);
            record.set_link(idx);
            if self.describe {
                let description = (&*joint_ref.description).clone();
                let msg = RillToServer::Description {
                    list: vec![description],
                };
                self.send_global(msg);
            }
        } else {
            log::error!("Provider for {} already registered.", path);
        }
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
                self.sender.set(sender);
                let entry_id = self.entry_id.clone();
                let msg = RillToServer::Declare { entry_id };
                self.send_global(msg);
            }
            WsClientStatus::Failed { reason } => {
                log::error!("Connection failed: {}", reason);
                // TODO: Try to reconnect...
                self.stop_all();
                self.describe = false;
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
                if let Some(idx) = self.index.find(&path).and_then(Record::get_link) {
                    if let Some(joint) = self.joints.get_mut(*idx) {
                        if active {
                            joint.subscribers.insert(direct_id);
                            // Send it before the flag switched on
                            let msg = RillToServer::BeginStream { snapshot: None };
                            self.sender.response(direct_id.into(), msg);
                            joint.try_switch_on();
                        } else {
                            joint.subscribers.remove(&direct_id);
                            joint.try_switch_off();
                            // Send it after the flag switched off
                            let msg = RillToServer::EndStream;
                            self.sender.response(direct_id.into(), msg);
                        }
                    } else {
                        log::error!("Inconsistent state of the storage: no Joint with the index {} of path {:?}", idx, path);
                    }
                } else {
                    log::warn!("Path not found: {:?}", path);
                    let reason = "path not found".to_string();
                    let msg = RillToServer::Error { reason };
                    self.sender.response(direct_id.into(), msg);
                }
            }
            RillToProvider::ListOf { path } => {
                self.send_list_for(direct_id, &path);
            }
            RillToProvider::Describe { active } => {
                // TODO: Check or use `Direction` here?
                if active && !self.describe && !self.joints.is_empty() {
                    // Send all exist paths
                    let list = self
                        .joints
                        .iter()
                        .map(|(_idx, joint)| (&*joint.description).clone())
                        .collect();
                    let msg = RillToServer::Description { list };
                    self.send_global(msg);
                }
                self.describe = active;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Consumer<DataEnvelope> for RillWorker {
    async fn handle(
        &mut self,
        envelope: DataEnvelope,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        match envelope {
            DataEnvelope::DataEvent {
                idx,
                timestamp,
                data,
            } => {
                if let Some(joint) = self.joints.get(idx) {
                    let timestamp = timestamp.duration_since(SystemTime::UNIX_EPOCH)?.into();
                    if !joint.subscribers.is_empty() {
                        let direction = Direction::from(&joint.subscribers);
                        let msg = RillToServer::Data { timestamp, data };
                        self.sender.response(direction, msg);
                    } else {
                        // Passive filtering in action:
                        // Never `Broasdcast` data events. If the `Data` message received
                        // for the empty subscribers list that means it was the late unprocessed
                        // data generated before the stream was deactivated.
                        // This late data has to be dropped.
                    }
                } else {
                    log::error!("No joint for index: {}", idx);
                }
            }
            DataEnvelope::EndStream { description } => {
                log::info!("Remove provider: {:?}", description.path);
                // It's the last message in the stream. Safe to remove it from joints.
                if let Some(pf_record) = self.index.remove(&description.path) {
                    // TODO: Use `Record::try_into()?` instead of `get_link`
                    if let Some(idx) = pf_record.get_link() {
                        if self.joints.contains(*idx) {
                            self.joints.remove(*idx);
                        // The thread that dropped the provider can not exists anymore.
                        // The switch message will never be delivered.
                        // Not needed to switch off: `joint.force_switch(false);`
                        } else {
                            log::error!("FATAL! Inconsistent state of the joints slab.");
                            // TODO: Return error here
                        }
                    } else {
                        log::error!("Attempt to remove not linked path record.");
                        // TODO: Return error here
                    }
                }
            }
        }
        Ok(())
    }
}

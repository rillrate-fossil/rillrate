use super::term;
use crate::pathfinder::{Pathfinder, Record};
use crate::protocol::{
    Direction, EntryId, EntryType, Envelope, Path, ProviderReqId, RillProtocol, RillToProvider,
    RillToServer, StreamType, WideEnvelope, PORT,
};
use crate::providers::provider::DataEnvelope;
use crate::state::{ControlEvent, ControlReceiver};
use anyhow::{anyhow, Error};
use async_trait::async_trait;
use meio::prelude::{
    ActionHandler, Actor, Consumer, Context, Eliminated, IdOf, InteractionHandler, InterruptedBy,
    StartedBy, System, Task,
};
use meio_connect::{
    client::{WsClient, WsClientStatus, WsSender},
    WsIncoming,
};
use slab::Slab;
use std::collections::HashSet;
use std::time::{Duration, SystemTime};
use tokio::sync::watch;

// TODO: Add `DirectionSet` that can give `Direction` value that depends
// of the 0,1,N items contained

#[tokio::main]
pub(crate) async fn entrypoint(
    entry_id: EntryId,
    rx: ControlReceiver,
    term_rx: term::Receiver,
) -> Result<(), Error> {
    let blocker = term_rx
        .blocker
        .lock()
        .map_err(|_| anyhow!("can't take termination blocker"))?;
    let mut handle = System::spawn(RillWorker::new(entry_id));
    handle.attach(rx);
    term_rx.notifier_rx.await?;
    System::interrupt(&mut handle)?;
    handle.join().await;
    drop(blocker);
    Ok(())
}

struct JointHolder {
    path: Path,
    active: watch::Sender<bool>,
    subscribers: HashSet<ProviderReqId>,
    stream_type: StreamType,
}

impl JointHolder {
    fn new(path: Path, active: watch::Sender<bool>, stream_type: StreamType) -> Self {
        Self {
            path,
            active,
            subscribers: HashSet::new(),
            stream_type,
        }
    }

    fn switch(&mut self, active: bool) {
        if let Err(err) = self.active.broadcast(active) {
            log::error!(
                "Can't switch the stream {} to {}: {}",
                self.path,
                active,
                err
            );
        }
    }
}

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

struct RillWorker {
    url: String,
    entry_id: EntryId,
    sender: RillSender,
    index: Pathfinder<usize>,
    joints: Slab<JointHolder>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Group {
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
        let link = format!("ws://127.0.0.1:{}/live/provider", PORT);
        Self {
            url: link,
            entry_id,
            sender: RillSender::default(),
            index: Pathfinder::default(),
            joints: Slab::new(),
        }
    }

    fn send_entry_id(&mut self) {
        let entry_id = self.entry_id.clone();
        let msg = RillToServer::Declare { entry_id };
        self.sender.response(Direction::broadcast(), msg);
    }

    fn send_list_for(&mut self, direct_id: ProviderReqId, path: &Path) {
        let msg;
        if let Some(records) = self.index.find(path).map(Record::list) {
            let entries = records
                .map(|(entry_id, idx)| {
                    let stream_type = idx
                        .and_then(|idx| {
                            self.joints
                                .get(*idx)
                                .map(|joint| EntryType::Stream(joint.stream_type))
                        })
                        .unwrap_or(EntryType::Container);
                    (entry_id, stream_type)
                })
                .collect();
            log::trace!("Entries list for {:?}: {:?}", path, entries);
            msg = RillToServer::Entries { entries };
        } else {
            log::trace!("No entry for {:?} to get a list", path);
            let reason = format!("entry not found");
            msg = RillToServer::Error { reason };
        }
        self.sender.response(direct_id.into(), msg);
    }

    fn stop_all(&mut self) {
        for (_, holder) in self.joints.iter_mut() {
            holder.switch(false);
        }
    }
}

#[async_trait]
impl StartedBy<System> for RillWorker {
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
impl InterruptedBy<System> for RillWorker {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        // TODO: Stop all streams and send errors to subscribers!
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl Eliminated<Task<WsClient<RillProtocol, Self>>> for RillWorker {
    async fn handle(
        &mut self,
        _id: IdOf<Task<WsClient<RillProtocol, Self>>>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        // TODO: Drop unfinished tasks
        Ok(())
    }
}

#[async_trait]
impl Consumer<ControlEvent> for RillWorker {
    async fn handle(&mut self, event: ControlEvent, ctx: &mut Context<Self>) -> Result<(), Error> {
        match event {
            ControlEvent::RegisterJoint {
                path,
                stream_type,
                joint,
                active,
                rx,
            } => {
                log::debug!("Creating provider with path: {:?}", path);
                let entry = self.joints.vacant_entry();
                let idx = entry.key();
                // TODO: How to return the idx without `Joint`?
                joint.assign(idx);
                let holder = JointHolder::new(path.clone(), active, stream_type);
                entry.insert(holder);
                ctx.address().attach(rx);
                self.index.dig(path).set_link(idx);
            }
        }
        Ok(())
    }
}

#[async_trait]
impl InteractionHandler<WsClientStatus<RillProtocol>> for RillWorker {
    async fn handle(
        &mut self,
        status: WsClientStatus<RillProtocol>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        match status {
            WsClientStatus::Connected { sender } => {
                self.sender.set(sender);
                self.send_entry_id();
            }
            WsClientStatus::Failed { reason } => {
                log::error!("Connection failed: {}", reason);
                // TODO: Try to reconnect...
                self.stop_all();
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
        log::trace!("Incoming request: {:?}", msg.0);
        let direct_id = msg.0.direct_id;
        match msg.0.data {
            RillToProvider::ListOf { path } => {
                self.send_list_for(direct_id.into(), &path);
            }
            RillToProvider::ControlStream { path, active } => {
                log::debug!("Switching the stream {:?} to {:?}", path, active);
                if let Some(idx) = self.index.find(&path).and_then(Record::get_link) {
                    if let Some(holder) = self.joints.get_mut(*idx) {
                        if active {
                            holder.subscribers.insert(direct_id);
                            // Send it before the flag switched on
                            let msg = RillToServer::BeginStream;
                            self.sender.response(direct_id.into(), msg);
                            holder.switch(true);
                        } else {
                            holder.subscribers.remove(&direct_id);
                            holder.switch(false);
                            // Send it after the flag switched off
                            let msg = RillToServer::EndStream;
                            self.sender.response(direct_id.into(), msg);
                        }
                    } else {
                        log::error!("Inconsistent state of the storage: no Joint with the index {} of path {:?}", idx, path);
                    }
                } else {
                    log::warn!("Path not found: {:?}", path);
                    let reason = format!("path not found");
                    let msg = RillToServer::Error { reason };
                    self.sender.response(direct_id.into(), msg);
                }
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
        if let Some(holder) = self.joints.get(envelope.idx) {
            if !holder.subscribers.is_empty() {
                let direction = Direction::from(&holder.subscribers);
                let timestamp = envelope.timestamp.duration_since(SystemTime::UNIX_EPOCH)?;
                let msg = RillToServer::Data {
                    timestamp,
                    data: envelope.data,
                };
                self.sender.response(direction, msg);
            } else {
                // Passive filtering in action:
                // Never `Broasdcast` data events. If the `Data` message received
                // for the empty subscribers list that means it was the late unprocessed
                // data generated before the stream was deactivated.
                // This late data has to be dropped.
            }
        }
        Ok(())
    }
}

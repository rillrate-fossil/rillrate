use crate::pathfinder::{Pathfinder, Record};
use crate::protocol::{
    DirectId, Direction, EntryId, Envelope, Path, RillProviderProtocol, RillToProvider,
    RillToServer, WideEnvelope, PORT,
};
use crate::provider::{DataEnvelope, Joint};
use crate::state::{ControlEvent, ControlReceiver};
use anyhow::Error;
use async_trait::async_trait;
use meio::{ActionHandler, Actor, Context, InteractionHandler, LiteTask, Supervisor};
use meio_connect::{
    client::{WsClient, WsClientStatus, WsSender},
    WsIncoming,
};
use slab::Slab;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

// TODO: Add `DirectionSet` that can give `Direction` value that depends
// of the 0,1,N items contained

#[tokio::main]
pub(crate) async fn entrypoint(entry_id: EntryId, rx: ControlReceiver) {
    let mut handle = RillWorker::new(entry_id).start(Supervisor::None);
    handle.attach(rx);
    handle.join().await;
}

struct JointHolder {
    joint: Arc<Joint>,
    subscribers: HashSet<DirectId>,
}

impl JointHolder {
    fn new(joint: Arc<Joint>) -> Self {
        Self {
            joint,
            subscribers: HashSet::new(),
        }
    }
}

#[derive(Default)]
struct RillSender {
    sender: Option<WsSender<WideEnvelope<RillToServer>>>,
}

impl RillSender {
    fn set(&mut self, sender: WsSender<WideEnvelope<RillToServer>>) {
        self.sender = Some(sender);
    }

    fn response(&mut self, direction: Direction, data: RillToServer) {
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

#[async_trait]
impl Actor for RillWorker {
    fn name(&self) -> String {
        format!("RillWorker({})", self.url)
    }

    async fn initialize(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let client = WsClient::new(
            self.url.clone(),
            Some(Duration::from_secs(1)),
            ctx.address().clone(),
        );
        let ws_client = client.start(ctx.bind());
        ctx.terminator().insert_to_single_stage(ws_client);
        Ok(())
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

    fn send_list_for(&mut self, direct_id: DirectId, path: &Path) {
        let entries = self
            .index
            .discover(path)
            .map(Record::list)
            .unwrap_or_default();
        log::trace!("Entries list for {:?}: {:?}", path, entries);
        let msg = RillToServer::Entries { entries };
        self.sender.response(direct_id.into(), msg);
    }

    fn stop_all(&mut self) {
        for (_, holder) in self.joints.iter_mut() {
            holder.joint.switch(false);
        }
    }
}

#[async_trait]
impl ActionHandler<ControlEvent> for RillWorker {
    async fn handle(&mut self, event: ControlEvent, ctx: &mut Context<Self>) -> Result<(), Error> {
        match event {
            ControlEvent::RegisterJoint {
                entry_id,
                joint,
                rx,
            } => {
                let entry = self.joints.vacant_entry();
                let idx = entry.key();
                joint.assign(idx);
                let holder = JointHolder::new(joint);
                entry.insert(holder);
                ctx.address().attach(rx);
                let path = Path::from(vec![self.entry_id.clone(), entry_id]);
                self.index.dig(path).set_link(idx);
            }
        }
        Ok(())
    }
}

#[async_trait]
impl InteractionHandler<WsClientStatus<RillProviderProtocol>> for RillWorker {
    async fn handle(
        &mut self,
        status: WsClientStatus<RillProviderProtocol>,
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
impl ActionHandler<WsIncoming<Envelope<RillToProvider>>> for RillWorker {
    async fn handle(
        &mut self,
        msg: WsIncoming<Envelope<RillToProvider>>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let direct_id = msg.0.direct_id;
        match msg.0.data {
            RillToProvider::ListOf { path } => {
                self.send_list_for(direct_id.into(), &path);
            }
            RillToProvider::ControlStream { path, active } => {
                log::debug!("Switching the stream {:?} to {:?}", path, active);
                if let Some(idx) = self.index.discover(&path).and_then(Record::get_link) {
                    if let Some(holder) = self.joints.get_mut(*idx) {
                        if active {
                            holder.subscribers.insert(direct_id);
                            // Send it before the flag switched on
                            let msg = RillToServer::BeginStream;
                            self.sender.response(direct_id.into(), msg);
                            holder.joint.switch(true);
                        } else {
                            holder.subscribers.remove(&direct_id);
                            holder.joint.switch(false);
                            // Send it after the flag switched off
                            let msg = RillToServer::EndStream;
                            self.sender.response(direct_id.into(), msg);
                        }
                    } else {
                        log::error!("Inconsistent state of the storage: no Joint with the index {} of path {:?}", idx, path);
                    }
                } else {
                    log::warn!("Path not found: {:?}", path);
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<DataEnvelope> for RillWorker {
    async fn handle(
        &mut self,
        envelope: DataEnvelope,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        if let Some(holder) = self.joints.get(envelope.idx) {
            if !holder.subscribers.is_empty() {
                let direction = Direction::from(&holder.subscribers);
                let msg = RillToServer::Data {
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

use crate::protocol::{
    DirectId, EntryId, Envelope, Path, RillProviderProtocol, RillToProvider, RillToServer, PORT,
};
use crate::provider::{DataEnvelope, Joint};
use crate::provider2;
use crate::state::{ControlEvent, ControlReceiver};
use anyhow::Error;
use async_trait::async_trait;
use meio::{ActionHandler, Actor, Context, InteractionHandler, LiteTask, Supervisor};
use meio_connect::{
    client::{WsClient, WsClientStatus, WsSender},
    WsIncoming,
};
use std::collections::{HashMap, HashSet};
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
    joint: Arc<provider2::Joint>,
    subscribers: HashSet<DirectId>,
}

impl JointHolder {
    fn new(joint: Arc<provider2::Joint>) -> Self {
        Self {
            joint,
            subscribers: HashSet::new(),
        }
    }
}

struct RillWorker {
    url: String,
    entry_id: EntryId,
    sender: Option<WsSender<Envelope<RillToServer>>>,
    joints: HashMap<EntryId, JointHolder>,
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
            sender: None,
            joints: HashMap::new(),
        }
    }

    fn response(&mut self, direct_id: DirectId, msg: RillToServer) {
        if let Some(sender) = self.sender.as_mut() {
            let envelope = Envelope {
                direct_id,
                data: msg,
            };
            sender.send(envelope);
        } else {
            //log::error!("Can't send a response. Not connected.");
        }
    }

    fn send_entry_id(&mut self) {
        let entry_id = self.entry_id.clone();
        let msg = RillToServer::Declare { entry_id };
        // TODO: Use `Direction::Broadcast` here.
        self.response(DirectId::from(0), msg);
    }

    fn send_list_for(&mut self, direct_id: DirectId, path: &Path) {
        let entries;
        match path.as_ref() {
            [provider] if *provider == self.entry_id => {
                entries = self.joints.keys().cloned().collect();
            }
            _ => {
                entries = Vec::new();
            }
        }
        let msg = RillToServer::Entries { entries };
        self.response(direct_id, msg);
    }

    fn stop_all(&mut self) {
        for holder in self.joints.values() {
            holder.joint.switch(false);
        }
    }
}

#[async_trait]
impl ActionHandler<ControlEvent> for RillWorker {
    async fn handle(&mut self, event: ControlEvent, ctx: &mut Context<Self>) -> Result<(), Error> {
        match event {
            ControlEvent::RegisterJoint { joint, rx } => {
                // TODO: Remove the branch!
            }
            ControlEvent::RegisterJoint2 {
                entry_id,
                joint,
                rx,
            } => {
                let holder = JointHolder::new(joint);
                self.joints.insert(entry_id, holder);
                ctx.address().attach(rx);
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
                self.sender = Some(sender);
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
        match msg.0.data {
            RillToProvider::ListOf { path } => {
                self.send_list_for(msg.0.direct_id, &path);
            }
            RillToProvider::ControlStream { path, active } => match path.as_ref() {
                [root, entry_id] if *root == self.entry_id => {
                    let direct_id = msg.0.direct_id;
                    if let Some(holder) = self.joints.get_mut(entry_id) {
                        if active {
                            holder.subscribers.insert(direct_id);
                        } else {
                            holder.subscribers.remove(&direct_id);
                        }
                        holder.joint.switch(active);
                    }
                }
                _ => {
                    todo!("subpaths not supported yet");
                }
            },
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
        let msg = RillToServer::Data {
            data: envelope.data,
        };
        // TODO: Drop the `Data` if no `DirectId`s remained
        // Passive filtering
        // TODO: Get the `Direction`
        //self.response(msg);
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<provider2::DataEnvelope> for RillWorker {
    async fn handle(
        &mut self,
        envelope: provider2::DataEnvelope,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let msg = RillToServer::Data {
            data: envelope.data,
        };
        // TODO: Drop the `Data` if no `DirectId`s remained
        // Passive filtering
        // TODO: Get the `Direction`
        //self.response(msg);
        Ok(())
    }
}

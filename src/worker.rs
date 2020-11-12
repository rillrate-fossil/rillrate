use super::{ControlEvent, ControlReceiver};
use crate::protocol::{EntryId, Path, RillProviderProtocol, RillToProvider, RillToServer, PORT};
use crate::provider::{DataEnvelope, Joint};
use anyhow::Error;
use async_trait::async_trait;
use meio::{ActionHandler, Actor, Context, InteractionHandler, LiteTask, Supervisor};
use meio_connect::{
    client::{WsClient, WsClientStatus, WsSender},
    WsIncoming,
};
use std::collections::HashMap;
use std::time::Duration;

#[tokio::main]
pub(crate) async fn entrypoint(entry_id: EntryId, rx: ControlReceiver) {
    let mut handle = RillWorker::new(entry_id).start(Supervisor::None);
    handle.attach(rx);
    handle.join().await;
}

struct RillWorker {
    url: String,
    entry_id: EntryId,
    sender: Option<WsSender<RillToServer>>,
    joints: HashMap<EntryId, Box<dyn Joint>>,
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

    fn response(&mut self, msg: RillToServer) {
        if let Some(sender) = self.sender.as_mut() {
            sender.send(msg);
        } else {
            //log::error!("Can't send a response. Not connected.");
        }
    }

    fn send_entry_id(&mut self) {
        let entry_id = self.entry_id.clone();
        let msg = RillToServer::Declare { entry_id };
        self.response(msg);
    }

    fn send_list_for(&mut self, path: &Path) {
        let entries = self.joints.keys().cloned().collect();
        let msg = RillToServer::Entries { entries };
        self.response(msg);
    }

    fn stop_all(&mut self) {
        for joint in self.joints.values() {
            joint.switch(false);
        }
    }
}

#[async_trait]
impl ActionHandler<ControlEvent> for RillWorker {
    async fn handle(&mut self, event: ControlEvent, ctx: &mut Context<Self>) -> Result<(), Error> {
        match event {
            ControlEvent::RegisterJoint { joint, rx } => {
                let entry_id = joint.entry_id().to_owned();
                self.joints.insert(entry_id, joint);
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
impl ActionHandler<WsIncoming<RillToProvider>> for RillWorker {
    async fn handle(
        &mut self,
        msg: WsIncoming<RillToProvider>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        match msg.0 {
            RillToProvider::ListOf { path } => {
                self.send_list_for(&path);
            }
            RillToProvider::ControlStream {
                entry_id,
                direct_id,
                active,
            } => {
                if let Some(joint) = self.joints.get(&entry_id) {
                    joint.switch(active);
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
        let msg = RillToServer::Data {
            direct_id: todo!(), //envelope.entry_id,
            data: envelope.data,
        };
        self.response(msg);
        Ok(())
    }
}

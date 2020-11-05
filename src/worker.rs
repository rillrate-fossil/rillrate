use super::{ControlEvent, ControlReceiver};
use crate::protocol::{Path, RillServerProtocol, RillToProvider, RillToServer, StreamId, PORT};
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
pub(crate) async fn entrypoint(rx: ControlReceiver) {
    let mut handle = RillWorker::new().start(Supervisor::None);
    handle.attach(rx);
    handle.join().await;
}

struct RillWorker {
    url: String,
    sender: Option<WsSender<RillToServer>>,
    joints: HashMap<StreamId, Box<dyn Joint>>,
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
        ctx.terminator()
            .new_stage("ws_client", false)
            .insert(ws_client);
        Ok(())
    }
}

impl RillWorker {
    pub fn new() -> Self {
        let link = format!("ws://127.0.0.1:{}/provider/io", PORT);
        Self {
            url: link,
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

    fn send_declarations(&mut self) {
        if !self.joints.is_empty() {
            let streams: Vec<_> = self
                .joints
                .iter()
                // TODO: Add `path` prefix
                .map(|(stream_id, provider)| (*stream_id, provider.path()))
                .collect();
            for (stream_id, path) in streams {
                self.send_declaration(stream_id, path);
            }
        }
    }

    fn send_declaration(&mut self, stream_id: StreamId, path: Path) {
        let msg = RillToServer::DeclareStream {
            stream_id,
            path: path.into(),
        };
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
                let stream_id = joint.stream_id();
                let path = joint.path();
                self.joints.insert(stream_id, joint);
                ctx.address().attach(rx);
                self.send_declaration(stream_id, path);
            }
        }
        Ok(())
    }
}

#[async_trait]
impl InteractionHandler<WsClientStatus<RillServerProtocol>> for RillWorker {
    async fn handle(
        &mut self,
        status: WsClientStatus<RillServerProtocol>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        match status {
            WsClientStatus::Connected { sender } => {
                self.sender = Some(sender);
                self.send_declarations();
            }
            WsClientStatus::Failed(_reason) => {
                // TODO: Log the reason
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
            RillToProvider::ControlStream { stream_id, active } => {
                if let Some(joint) = self.joints.get(&stream_id) {
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
            stream_id: envelope.stream_id,
            data: envelope.data,
        };
        self.response(msg);
        Ok(())
    }
}

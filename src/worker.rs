use super::{ControlEvent, ControlReceiver};
use crate::protocol::{Path, RillServerProtocol, RillToProvider, RillToServer, StreamId, PORT};
use crate::provider::{DataReceiver, ProviderCell};
use anyhow::Error;
use async_trait::async_trait;
use meio::{ActionHandler, Actor, Context, InteractionHandler, LiteTask, Supervisor};
use meio_ws::{
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

struct StreamRecord {
    provider: &'static ProviderCell,
    stream_id: StreamId,
}

struct RillWorker {
    url: String,
    sender: Option<WsSender<RillToServer>>,

    declared_streams: Vec<StreamRecord>,
    initial_streams: HashMap<StreamId, DataReceiver>,
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
            declared_streams: Vec::new(),
            initial_streams: HashMap::new(),
        }
    }
}

#[async_trait]
impl ActionHandler<ControlEvent> for RillWorker {
    async fn handle(&mut self, event: ControlEvent, _ctx: &mut Context<Self>) -> Result<(), Error> {
        match event {
            ControlEvent::RegisterStream {
                provider,
                initial_receiver,
            } => {
                let stream_id = StreamId(self.declared_streams.len() as u64);
                let record = StreamRecord {
                    provider,
                    stream_id,
                };
                self.declared_streams.push(record);
                self.initial_streams.insert(stream_id, initial_receiver);
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
                // TODO: Send declarations if they already exists
            }
            WsClientStatus::Failed(reason) => {}
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
            RillToProvider::CanDrop { stream_id } => {
                todo!();
            }
            RillToProvider::ControlStream { stream_id, active } => {
                todo!();
            }
        }
        Ok(())
    }
}

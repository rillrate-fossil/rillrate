use super::{ControlEvent, ControlReceiver};
use crate::protocol::{Path, RillServerProtocol, RillToProvider, RillToServer, StreamId, PORT};
use crate::provider::{Data, DataReceiver, ProviderCell};
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
    collector: Option<Vec<Data>>,
}

enum ProcessingRule {
    Collect { pool: Vec<Data> },
    Forward,
    Block,
}

struct RillWorker {
    url: String,
    sender: Option<WsSender<RillToServer>>,
    declared_streams: HashMap<StreamId, StreamRecord>,
    // TODO: Fill with `Collect` values for the all unknown streams
    rules: Vec<ProcessingRule>,
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
            declared_streams: HashMap::new(),
            rules: Vec::new(),
        }
    }
}

#[async_trait]
impl ActionHandler<ControlEvent> for RillWorker {
    async fn handle(&mut self, event: ControlEvent, ctx: &mut Context<Self>) -> Result<(), Error> {
        match event {
            ControlEvent::RegisterStream { provider, rx } => {
                let stream_id = provider.stream_id();
                let record = StreamRecord {
                    provider,
                    // Attach the collector to store initial records before the server
                    // will decide what to do with them.
                    collector: Some(Vec::new()),
                };
                self.declared_streams.insert(stream_id, record);
                ctx.address().attach(rx);
                if let Some(sender) = self.sender.as_mut() {
                    // TODO: Send a declaration of the stream if there is a connection
                }
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
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        match msg.0 {
            RillToProvider::CanDrop { stream_id } => {
                if let Some(record) = self.declared_streams.get_mut(&stream_id) {
                    record.collector.take();
                }
            }
            RillToProvider::ControlStream { stream_id, active } => {
                if let Some(record) = self.declared_streams.get(&stream_id) {
                    record.provider.switch(active);
                }
            }
        }
        Ok(())
    }
}

// TODO: Add `StreamId` here...
#[async_trait]
impl ActionHandler<Data> for RillWorker {
    async fn handle(&mut self, data: Data, ctx: &mut Context<Self>) -> Result<(), Error> {
        todo!();
    }
}

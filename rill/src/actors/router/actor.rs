use crate::actors::snapshot::SnapshotLink;
use crate::actors::supervisor::RillSupervisor;
use crate::config::RillConfig;
use crate::state::{TracerMode, UpgradeStateEvent};
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
use rill_protocol::provider::{
    Description, Direction, EntryType, Envelope, Path, ProviderReqId, RillEvent, RillProtocol,
    RillToProvider, RillToServer, WideEnvelope,
};
use std::time::{Duration, SystemTime};

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

/// Actor that connects to a server and routes requests.
pub struct RillRouter {
    config: RillConfig,
    sender: RillSender,
    snapshot: SnapshotLink,
}

impl RillRouter {
    pub fn new(config: RillConfig, snapshot: SnapshotLink) -> Self {
        Self {
            config,
            sender: RillSender::default(),
            snapshot,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    WsConnection,
    Streams,
}

#[async_trait]
impl Actor for RillRouter {
    type GroupBy = Group;

    fn name(&self) -> String {
        format!("RillRouter({})", self.config.url())
    }
}

impl RillRouter {
    fn send_global(&mut self, msg: RillToServer) {
        self.sender.response(Direction::broadcast(), msg);
    }
}

#[async_trait]
impl StartedBy<RillSupervisor> for RillRouter {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![Group::WsConnection, Group::Streams]);
        let client = WsClient::new(
            self.config.url().to_string(),
            Some(Duration::from_secs(1)),
            ctx.address().clone(),
        );
        ctx.spawn_task(client, Group::WsConnection);
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<RillSupervisor> for RillRouter {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<WsClient<RillProtocol, Self>> for RillRouter {
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
impl InstantActionHandler<WsClientStatus<RillProtocol>> for RillRouter {
    async fn handle(
        &mut self,
        status: WsClientStatus<RillProtocol>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        match status {
            WsClientStatus::Connected { sender } => {
                self.sender.set(sender);
                let entry_id = self.config.entry_id().clone();
                let msg = RillToServer::Declare { entry_id };
                self.send_global(msg);
            }
            WsClientStatus::Failed { reason } => {
                log::error!("Connection failed: {}", reason);
                // TODO: Try to reconnect...
                // TODO: self.stop_all();
                // TODO: self.describe = false;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<WsIncoming<Envelope<RillProtocol, RillToProvider>>> for RillRouter {
    async fn handle(
        &mut self,
        msg: WsIncoming<Envelope<RillProtocol, RillToProvider>>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl Consumer<UpgradeStateEvent> for RillRouter {
    fn stream_group(&self) -> Group {
        Group::Streams
    }

    async fn handle(
        &mut self,
        chunk: Vec<UpgradeStateEvent>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        for event in chunk {
            match event {
                UpgradeStateEvent::RegisterTracer {
                    description,
                    realtime,
                    snapshot,
                    storage,
                } => {
                    if let Some(flow) = realtime {
                        // TODO: Attach to RealtimeWorker
                    }
                    if let Some(flow) = snapshot {
                        self.snapshot
                            .attach_tracer(description, flow.receiver)
                            .await?;
                    }
                    if let Some(flow) = storage {
                        // TODO: Attach to SnapshotWorker
                    }
                }
            }
        }
        Ok(())
    }
}

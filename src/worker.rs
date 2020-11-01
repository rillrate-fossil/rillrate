use super::{ControlEvent, ControlReceiver};
use crate::protocol::{Path, StreamId};
use crate::provider::{DataReceiver, ProviderCell};
use anyhow::Error;
use async_trait::async_trait;
use meio::{ActionHandler, Actor, Context, Supervisor};
use std::collections::HashMap;

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
    declared_streams: Vec<StreamRecord>,
    initial_streams: HashMap<StreamId, DataReceiver>,
}

impl Actor for RillWorker {}

impl RillWorker {
    pub fn new() -> Self {
        Self {
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

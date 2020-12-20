use crate::actors::supervisor::RillSupervisor;
use crate::exporters::BroadcastData;
use crate::protocol::{Path, RillData};
use anyhow::Error;
use async_trait::async_trait;
use futures::StreamExt;
use meio::prelude::{
    task::{HeartBeat, Tick},
    ActionHandler, Actor, Address, Context, Eliminated, IdOf, Interaction, InteractionHandler,
    InterruptedBy, LiteTask, StartedBy, StopReceiver, Task, TryConsumer,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

pub struct GraphiteExporter {
    rx: Option<broadcast::Receiver<Arc<BroadcastData>>>,
    metrics: HashMap<Path, RillData>,
}

impl GraphiteExporter {
    pub fn new(receiver: broadcast::Receiver<Arc<BroadcastData>>) -> Self {
        Self {
            rx: Some(receiver),
            metrics: HashMap::new(),
        }
    }
}

impl Actor for GraphiteExporter {
    type GroupBy = ();
}

#[async_trait]
impl StartedBy<RillSupervisor> for GraphiteExporter {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        let rx = self
            .rx
            .take()
            .ok_or(Error::msg(
                "attempt to start the same graphite exporter twice",
            ))?
            .into_stream()
            .boxed();
        ctx.address().attach(rx);
        let heartbeat = HeartBeat::new(Duration::from_millis(1_000), ctx.address().clone());
        ctx.spawn_task(heartbeat, ());
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<RillSupervisor> for GraphiteExporter {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl Eliminated<Task<HeartBeat>> for GraphiteExporter {
    async fn handle(
        &mut self,
        _id: IdOf<Task<HeartBeat>>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<Tick> for GraphiteExporter {
    async fn handle(&mut self, _: Tick, ctx: &mut Context<Self>) -> Result<(), Error> {
        for (path, data) in self.metrics.drain() {
            // TODO: Render lines and send them pickled
        }
        Ok(())
    }
}

#[async_trait]
impl TryConsumer<Arc<BroadcastData>> for GraphiteExporter {
    type Error = broadcast::RecvError;

    async fn handle(
        &mut self,
        item: Arc<BroadcastData>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        self.metrics.insert(item.path.clone(), item.data.clone());
        Ok(())
    }

    async fn error(&mut self, err: Self::Error, ctx: &mut Context<Self>) -> Result<(), Error> {
        log::error!(
            "Broadcasting stream failed. Not possible to continue: {}",
            err
        );
        ctx.shutdown();
        Ok(())
    }
}

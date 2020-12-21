use crate::actors::supervisor::RillSupervisor;
use crate::exporters::ExportEvent;
use crate::protocol::{Path, RillData};
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    task::{HeartBeat, Tick},
    ActionHandler, Actor, Context, Eliminated, IdOf, InterruptedBy, StartedBy, Task, TryConsumer,
};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::broadcast;

pub struct GraphiteExporter {
    metrics: HashMap<Path, RillData>,
}

impl GraphiteExporter {
    pub fn new() -> Self {
        Self {
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
impl TryConsumer<ExportEvent> for GraphiteExporter {
    type Error = broadcast::RecvError;

    async fn handle(&mut self, event: ExportEvent, _ctx: &mut Context<Self>) -> Result<(), Error> {
        match event {
            ExportEvent::SetInfo { .. } => {
            }
            ExportEvent::BroadcastData { path, data, .. } => {
                self.metrics.insert(path, data);
            }
        }
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

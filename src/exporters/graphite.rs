use crate::actors::supervisor::RillSupervisor;
use crate::exporters::BroadcastData;
use anyhow::Error;
use async_trait::async_trait;
use futures::StreamExt;
use meio::prelude::{
    Actor, Address, Context, IdOf, Interaction, InteractionHandler, InterruptedBy, LiteTask,
    StartedBy, StopReceiver, Task, TryConsumer,
};
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct GraphiteExporter {
    rx: Option<broadcast::Receiver<Arc<BroadcastData>>>,
}

impl GraphiteExporter {
    pub fn new(receiver: broadcast::Receiver<Arc<BroadcastData>>) -> Self {
        Self { rx: Some(receiver) }
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
impl TryConsumer<Arc<BroadcastData>> for GraphiteExporter {
    type Error = broadcast::RecvError;

    async fn handle(
        &mut self,
        item: Arc<BroadcastData>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        todo!("send rendered line into a graphite connection");
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

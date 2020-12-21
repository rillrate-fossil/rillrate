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
use std::convert::TryInto;
use std::io::Write;
use std::time::Duration;
use tokio::sync::broadcast;

struct Record {
    timestamp: Duration,
    data: RillData,
}

pub struct GraphiteExporter {
    pickled: bool,
    metrics: HashMap<Path, Record>,
}

impl GraphiteExporter {
    pub fn new() -> Self {
        Self {
            pickled: true,
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
        if self.pickled {
            // Collect all metrics values into a pool
            let mut pool = Vec::with_capacity(self.metrics.len());
            for (path, record) in self.metrics.drain() {
                let converted: Result<f64, _> = record.data.try_into();
                match converted {
                    Ok(value) => {
                        let line = (path.to_string(), (record.timestamp.as_secs(), value));
                        pool.push(value);
                    }
                    Err(err) => {
                        log::error!("Can't send {} to the Graphite: {}", path, err);
                    }
                }
            }
            // Serialize with pickle
            let mut buffer = Vec::new();
            buffer.write(&0_u32.to_be_bytes())?;
            serde_pickle::to_writer(&mut buffer, &pool, false)?;
            let prefix_len = std::mem::size_of::<u32>();
            let len: u32 = (buffer.len() - prefix_len).try_into()?;
            buffer[0..prefix_len].copy_from_slice(&len.to_be_bytes());
        } else {
            // TODO: Support the plain-text format
        }
        Ok(())
    }
}

#[async_trait]
impl TryConsumer<ExportEvent> for GraphiteExporter {
    type Error = broadcast::RecvError;

    async fn handle(&mut self, event: ExportEvent, _ctx: &mut Context<Self>) -> Result<(), Error> {
        match event {
            ExportEvent::SetInfo { .. } => {}
            ExportEvent::BroadcastData {
                path,
                data,
                timestamp,
            } => {
                let record = Record { timestamp, data };
                self.metrics.insert(path, record);
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

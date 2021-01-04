use super::{ExportEvent, ExporterLinkForClient};
use crate::actors::exporter::Exporter;
use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{
    task::{HeartBeat, Tick},
    ActionHandler, Actor, Context, IdOf, InterruptedBy, LiteTask, StartedBy, TaskEliminated,
    TaskError, TryConsumer,
};
use rill_protocol::provider::{Path, RillData};
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::Write;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::broadcast;

struct Record {
    timestamp: Duration,
    data: RillData,
}

pub struct GraphiteExporter {
    exporter: ExporterLinkForClient,
    pickled: bool,
    metrics: HashMap<Path, Record>,
    sender: broadcast::Sender<Vec<u8>>,
}

impl GraphiteExporter {
    pub fn new(exporter: ExporterLinkForClient) -> Self {
        let (sender, _rx) = broadcast::channel(32);
        Self {
            exporter,
            pickled: true,
            metrics: HashMap::new(),
            sender,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    HeartBeat,
    Connection,
}

impl Actor for GraphiteExporter {
    type GroupBy = Group;
}

#[async_trait]
impl StartedBy<Exporter> for GraphiteExporter {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![Group::HeartBeat, Group::Connection]);
        let heartbeat = HeartBeat::new(Duration::from_millis(1_000), ctx.address().clone());
        ctx.spawn_task(heartbeat, Group::HeartBeat);
        let connection = Connection::new(self.sender.clone());
        ctx.spawn_task(connection, Group::Connection);
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<Exporter> for GraphiteExporter {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<HeartBeat> for GraphiteExporter {
    async fn handle(
        &mut self,
        _id: IdOf<HeartBeat>,
        _result: Result<(), TaskError>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<Connection> for GraphiteExporter {
    async fn handle(
        &mut self,
        _id: IdOf<Connection>,
        _result: Result<(), TaskError>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<Tick> for GraphiteExporter {
    async fn handle(&mut self, _: Tick, _ctx: &mut Context<Self>) -> Result<(), Error> {
        if self.sender.receiver_count() > 0 {
            if self.pickled {
                // Collect all metrics values into a pool
                let mut pool = Vec::with_capacity(self.metrics.len());
                for (path, record) in self.metrics.drain() {
                    let converted: Result<f64, _> = record.data.try_into();
                    match converted {
                        Ok(value) => {
                            let line = (path.to_string(), (record.timestamp.as_secs(), value));
                            log::trace!("Graphite export: {} - {}", path, value);
                            pool.push(line);
                        }
                        Err(err) => {
                            log::error!("Can't send {} to the Graphite: {}", path, err);
                        }
                    }
                }
                // Serialize with pickle
                let mut buffer = Vec::new();
                Write::write(&mut buffer, &0_u32.to_be_bytes())?;
                serde_pickle::to_writer(&mut buffer, &pool, false)?;
                let prefix_len = std::mem::size_of::<u32>();
                let len: u32 = (buffer.len() - prefix_len).try_into()?;
                buffer[0..prefix_len].copy_from_slice(&len.to_be_bytes());
                self.sender.send(buffer).map_err(|_| {
                    Error::msg("Can't send data to Graphite (no active connections)")
                })?;
            } else {
                // TODO: Support the plain-text format
            }
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

struct Connection {
    sender: broadcast::Sender<Vec<u8>>,
}

impl Connection {
    fn new(sender: broadcast::Sender<Vec<u8>>) -> Self {
        Self { sender }
    }
}

#[async_trait]
impl LiteTask for Connection {
    type Output = ();

    async fn repeatable_routine(&mut self) -> Result<Self::Output, Error> {
        let mut rx = self.sender.subscribe();
        loop {
            // To reuse connection put this line up (outside of the loop and above `subscribe` call)
            let mut socket = TcpStream::connect("127.0.0.1:2004").await?;
            let data = rx.recv().await?;
            socket.write_all(&data).await?;
        }
    }
}

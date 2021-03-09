use super::{Observer, Publisher, SharedRecord};
use crate::actors::export::RillExport;
use crate::config::GraphiteConfig;
use anyhow::Error;
use async_trait::async_trait;
use meio::{
    task::{HeartBeat, Tick},
    ActionHandler, Actor, Context, IdOf, InterruptedBy, LiteTask, StartedBy, TaskEliminated,
    TaskError,
};
use meio_connect::server::HttpServerLink;
use rill_client::actors::broadcaster::{BroadcasterLinkForClient, PathNotification};
use rill_client::actors::client::ClientLink;
use rill_protocol::io::provider::{Path, PathPattern};
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::Write;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::broadcast;

pub struct GraphitePublisher {
    config: GraphiteConfig,
    broadcaster: BroadcasterLinkForClient,
    client: ClientLink,
    pickled: bool,
    metrics: HashMap<Path, SharedRecord>,
    sender: broadcast::Sender<Vec<u8>>,
}

impl Publisher for GraphitePublisher {
    type Config = GraphiteConfig;

    fn create(
        config: Self::Config,
        broadcaster: BroadcasterLinkForClient,
        client: ClientLink,
        _server: &HttpServerLink,
    ) -> Self {
        let (sender, _rx) = broadcast::channel(32);
        Self {
            config,
            broadcaster,
            client,
            pickled: true, // TODO: Get from the config
            metrics: HashMap::new(),
            sender,
        }
    }
}

impl GraphitePublisher {
    async fn graceful_shutdown(&mut self, ctx: &mut Context<Self>) {
        // TODO: Do this for the client
        //self.broadcaster.unsubscribe_all(ctx.address()).await.ok();
        ctx.shutdown();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Group {
    HeartBeat,
    Streams,
    Connection,
}

impl Actor for GraphitePublisher {
    type GroupBy = Group;
}

#[async_trait]
impl StartedBy<RillExport> for GraphitePublisher {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(vec![Group::HeartBeat, Group::Streams, Group::Connection]);
        let interval = self.config.interval.unwrap_or(1_000);
        let duration = Duration::from_millis(interval);
        let heartbeat = HeartBeat::new(duration, ctx.address().clone());
        ctx.spawn_task(heartbeat, Group::HeartBeat);
        let connection = Connection::new(self.sender.clone());
        ctx.spawn_task(connection, Group::Connection);
        self.broadcaster
            .subscribe_to_struct_changes(ctx.address().clone())
            .await?;
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<RillExport> for GraphitePublisher {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.graceful_shutdown(ctx).await;
        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<HeartBeat> for GraphitePublisher {
    async fn handle(
        &mut self,
        _id: IdOf<HeartBeat>,
        _result: Result<(), TaskError>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        self.graceful_shutdown(ctx).await;
        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<Connection> for GraphitePublisher {
    async fn handle(
        &mut self,
        _id: IdOf<Connection>,
        _result: Result<(), TaskError>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        self.graceful_shutdown(ctx).await;
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<Tick> for GraphitePublisher {
    async fn handle(&mut self, _: Tick, _ctx: &mut Context<Self>) -> Result<(), Error> {
        if self.sender.receiver_count() > 0 {
            if self.pickled {
                // Collect all metrics values into a pool
                let mut pool = Vec::with_capacity(self.metrics.len());
                for (path, record) in self.metrics.drain() {
                    if let Some(record) = record.get().await {
                        let ts = record.timestamp;
                        let value = record.value;
                        let line = (path.to_string(), (ts.as_secs(), value));
                        log::trace!("Graphite export: {} - {}", path, value);
                        pool.push(line);
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
impl ActionHandler<PathNotification> for GraphitePublisher {
    async fn handle(
        &mut self,
        msg: PathNotification,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        match msg {
            PathNotification::Paths { descriptions } => {
                for description in descriptions {
                    let path = &description.path;
                    // TODO: Improve that... Maybe use `PatternMatcher` that wraps `HashSet` of `Patterns`
                    let pattern = PathPattern { path: path.clone() };
                    if self.config.paths.contains(&pattern) {
                        let record = SharedRecord::new();
                        self.metrics.insert(path.clone(), record.clone());
                        let observer = Observer::new(description, self.client.clone(), record);
                        ctx.spawn_task(observer, Group::Streams);
                    }
                }
            }
            PathNotification::Name { .. } => {}
        }
        Ok(())
    }
}

#[async_trait]
impl TaskEliminated<Observer> for GraphitePublisher {
    async fn handle(
        &mut self,
        _id: IdOf<Observer>,
        _result: Result<(), TaskError>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
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

    async fn repeatable_routine(&mut self) -> Result<Option<Self::Output>, Error> {
        let mut rx = self.sender.subscribe();
        // TODO: Make url configurable
        loop {
            // To reuse connection put this line up (outside of the loop and above `subscribe` call)
            let mut socket = TcpStream::connect("127.0.0.1:2004").await?;
            let data = rx.recv().await?;
            socket.write_all(&data).await?;
        }
    }
}

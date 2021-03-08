use super::Publisher;
use crate::actors::export::RillExport;
use crate::config::GraphiteConfig;
use crate::publishers::converter::Extractor;
use anyhow::Error;
use async_trait::async_trait;
use futures::StreamExt;
use meio::{
    task::{HeartBeat, Tick},
    ActionHandler, Actor, Consumer, Context, IdOf, InterruptedBy, LiteTask, StartedBy,
    StreamAcceptor, TaskEliminated, TaskError,
};
use meio_connect::server::HttpServerLink;
use rill_client::actors::broadcaster::{BroadcasterLinkForClient, PathNotification};
use rill_client::actors::client::{ClientLink, StateOrDelta};
use rill_protocol::io::provider::{Path, PathPattern, RillEvent, Timestamp};
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::broadcast;

struct Record {
    extractor: Box<dyn Extractor>,
}

pub struct GraphitePublisher {
    config: GraphiteConfig,
    broadcaster: BroadcasterLinkForClient,
    client: ClientLink,
    pickled: bool,
    metrics: HashMap<Path, Record>,
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
                    if let Some((ts, value)) = record.extractor.to_value() {
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
                        let subscription =
                            self.client.subscribe_to_path(path.clone()).recv().await?;
                        let extractor = Extractor::make_extractor(&description);
                        let record = Record { extractor };
                        let path = Arc::new(path.clone());
                        let rx = subscription.map(move |item| (path.clone(), item));
                        ctx.attach(rx, Group::Streams);
                    }
                }
            }
            PathNotification::Name { .. } => {}
        }
        Ok(())
    }
}

#[async_trait]
impl Consumer<(Arc<Path>, StateOrDelta)> for GraphitePublisher {
    async fn handle(
        &mut self,
        (path, msg): (Arc<Path>, StateOrDelta),
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        todo!();
        //let value = extract_value(msg)?;
        /*
        if let Some(event) = chunk.into_iter().last() {
            let val: Result<f64, _> = event.data.try_into();
            match val {
                Ok(value) => {
                    let record = Record {
                        timestamp: event.timestamp,
                        value,
                    };
                    self.metrics.insert((&*path).clone(), record);
                }
                Err(err) => {
                    log::error!("Can't convert {} to a value: {}", path, err);
                }
            }
        }
        Ok(())
        */
    }
}

impl StreamAcceptor<Vec<RillEvent>> for GraphitePublisher {
    fn stream_group(&self) -> Self::GroupBy {
        Group::Streams
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

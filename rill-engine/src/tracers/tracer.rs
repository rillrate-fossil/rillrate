//! This module contains a generic `Tracer`'s methods.
use crate::state::RILL_LINK;
use anyhow::Error;
use futures::channel::mpsc;
use meio::Action;
use rill_protocol::flow::data::{self, TimedEvent};
use rill_protocol::io::provider::{Description, Path, Timestamp};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, SystemTime};
use tokio::sync::watch;

#[derive(Debug)]
pub(crate) enum DataEnvelope<T: data::Flow> {
    Event(TimedEvent<T::Event>),
}

impl<T: data::Flow> Action for DataEnvelope<T> {}

impl<T: data::Flow> DataEnvelope<T> {
    pub fn into_inner(self) -> TimedEvent<T::Event> {
        match self {
            Self::Event(event) => event,
        }
    }
}

// TODO: Remove that aliases and use raw types receivers in recorders.
pub(crate) type DataSender<T> = mpsc::UnboundedSender<DataEnvelope<T>>;
pub(crate) type DataReceiver<T> = mpsc::UnboundedReceiver<DataEnvelope<T>>;

pub(crate) enum TracerMode<T: data::Flow> {
    /// Real-time mode
    Push {
        state: T::State,
        receiver: Option<DataReceiver<T>>,
    },
    Pull {
        state: Weak<Mutex<T::State>>,
        interval: Duration,
    },
}

#[derive(Debug)]
enum InnerMode<T: data::Flow> {
    Push { sender: DataSender<T> },
    Pull { state: Arc<Mutex<T::State>> },
}

// TODO: Or require `Clone` for the `Flow` to derive this
impl<T: data::Flow> Clone for InnerMode<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Push { sender } => Self::Push {
                sender: sender.clone(),
            },
            Self::Pull { state } => Self::Pull {
                state: state.clone(),
            },
        }
    }
}

#[derive(Debug)]
pub(crate) struct TracerDescription<T> {
    pub path: Path,
    pub info: String,
    pub metric: T,
}

// TODO: Change to `TryInto`?
impl<T: data::Flow> TracerDescription<T> {
    /// Converts `TracerDescription` into a `Description`.
    pub fn to_description(&self) -> Result<Description, Error> {
        let metadata = self.metric.pack_metric()?;
        Ok(Description {
            path: self.path.clone(),
            info: self.info.clone(),
            stream_type: T::stream_type(),
            metadata,
        })
    }
}

/// The generic provider that forwards metrics to worker and keeps a flag
/// for checking the activitiy status of the `Tracer`.
#[derive(Debug)]
pub struct Tracer<T: data::Flow> {
    /// The receiver that used to activate/deactivate streams.
    active: watch::Receiver<bool>,
    description: Arc<TracerDescription<T>>,
    mode: InnerMode<T>,
}

impl<T: data::Flow> Clone for Tracer<T> {
    fn clone(&self) -> Self {
        Self {
            active: self.active.clone(),
            description: self.description.clone(),
            mode: self.mode.clone(),
        }
    }
}

impl<T: data::Flow> Tracer<T> {
    /// Creates a new `Tracer`.
    pub fn new(metric: T, state: T::State, path: Path, pull: Option<Duration>) -> Self {
        let stream_type = T::stream_type();
        let info = format!("{} - {}", path, stream_type);
        let description = TracerDescription { path, info, metric };
        // TODO: Remove this active watch channel?
        let (_active_tx, active_rx) = watch::channel(true);
        log::trace!("Creating Tracer with path: {:?}", description.path);
        let description = Arc::new(description);
        let inner_mode;
        let mode;
        if let Some(interval) = pull {
            let state = Arc::new(Mutex::new(state));
            mode = TracerMode::Pull {
                state: Arc::downgrade(&state),
                interval,
            };
            inner_mode = InnerMode::Pull { state };
        } else {
            let (tx, rx) = mpsc::unbounded();
            mode = TracerMode::Push {
                state,
                receiver: Some(rx),
            };
            inner_mode = InnerMode::Push { sender: tx };
        }
        let this = Tracer {
            active: active_rx,
            description: description.clone(),
            mode: inner_mode,
        };
        if let Err(err) = RILL_LINK.register_tracer(description, mode) {
            log::error!(
                "Can't register a Tracer. The worker can be terminated already: {}",
                err
            );
        }
        this
    }

    /// Returns a reference to a `Path` of the `Tracer`.
    pub fn path(&self) -> &Path {
        &self.description.path
    }

    /// Send an event to a `Recorder`.
    pub fn send(&self, data: T::Event, opt_system_time: Option<SystemTime>) {
        if self.is_active() {
            let ts = opt_system_time
                .unwrap_or_else(SystemTime::now)
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(Timestamp::from);
            match ts {
                Ok(timestamp) => {
                    let timed_event = TimedEvent {
                        timestamp,
                        event: data,
                    };
                    match &self.mode {
                        InnerMode::Push { sender } => {
                            let envelope = DataEnvelope::Event(timed_event);
                            // And will never send an event
                            if let Err(err) = sender.unbounded_send(envelope) {
                                log::error!("Can't transfer data to sender: {}", err);
                            }
                        }
                        InnerMode::Pull { state } => match state.lock() {
                            Ok(ref mut state) => {
                                self.description.metric.apply(state, timed_event);
                            }
                            Err(err) => {
                                log::error!("Can't lock the mutex to apply the changes: {}", err);
                            }
                        },
                    }
                }
                Err(err) => {
                    log::error!("Can't make a timestamp from provided system time: {}", err);
                }
            }
        }
    }
}

impl<T: data::Flow> Tracer<T> {
    /// Returns `true` is the `Tracer` has to send data.
    pub fn is_active(&self) -> bool {
        *self.active.borrow()
    }

    /* TODO: Remove or replace with an alternative
    /// Use this method to detect when stream had activated.
    ///
    /// It's useful if you want to spawn async coroutine that
    /// can read a batch of data, but will wait when some streams
    /// will be activated to avoid resources wasting.
    ///
    /// When the generating coroutine active you can use `is_active`
    /// method to detect when to change it to awaiting state again.
    pub async fn when_activated(&mut self) -> Result<(), Error> {
        loop {
            if self.is_active() {
                break;
            }
            self.active.changed().await?;
        }
        Ok(())
    }
    */
}

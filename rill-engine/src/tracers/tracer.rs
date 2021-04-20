//! This module contains a generic `Tracer`'s methods.
use crate::state::RILL_LINK;
use anyhow::Error;
use futures::channel::mpsc;
use meio::Action;
use rill_protocol::flow::core::{self, TimedEvent};
use rill_protocol::io::provider::{Description, Path, Timestamp};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, SystemTime};
use tokio::sync::{broadcast, watch};

#[derive(Debug)]
pub(crate) enum DataEnvelope<T: core::Flow> {
    Event(TimedEvent<T::Event>),
}

impl<T: core::Flow> Action for DataEnvelope<T> {}

impl<T: core::Flow> DataEnvelope<T> {
    pub fn into_inner(self) -> TimedEvent<T::Event> {
        match self {
            Self::Event(event) => event,
        }
    }
}

// TODO: Remove that aliases and use raw types receivers in recorders.
pub(crate) type DataSender<T> = mpsc::UnboundedSender<DataEnvelope<T>>;
pub(crate) type DataReceiver<T> = mpsc::UnboundedReceiver<DataEnvelope<T>>;

pub type Watcher<T> = broadcast::Receiver<T>;

pub(crate) enum TracerMode<T: core::Flow> {
    /* TODO: THE Idea to implement storage:
     *
     * Routed recorder shares the state and listens requests to forward them
     * and return responses.
     *
     * Routed {
     *   initial_state: T, - aka lazy state / bootstrap state
     *   interactor: Address<?> or spawned routine, - to send requests there to update individual states
     * },
     */
    /// Real-time mode
    Push {
        state: T,
        receiver: Option<DataReceiver<T>>,
    },
    /// Pulling for intensive streams with high-load activities
    Pull {
        state: Weak<Mutex<T>>,
        interval: Duration,
    },
    /// Used for controls
    Watched {
        state: T,
        /// For sending events to a `Tracer` instances
        sender: broadcast::Sender<T::Event>,
    },
}

#[derive(Debug)]
enum InnerMode<T: core::Flow> {
    Push {
        sender: DataSender<T>,
    },
    Pull {
        state: Arc<Mutex<T>>,
    },
    Watched {
        /// Kept for generating new `Receiver`s
        sender: Arc<broadcast::Sender<T::Event>>,
    },
}

// TODO: Or require `Clone` for the `Flow` to derive this
impl<T: core::Flow> Clone for InnerMode<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Push { sender } => Self::Push {
                sender: sender.clone(),
            },
            Self::Pull { state } => Self::Pull {
                state: state.clone(),
            },
            Self::Watched { sender, .. } => Self::Watched {
                sender: sender.clone(),
            },
        }
    }
}

/// The generic provider that forwards metrics to worker and keeps a flag
/// for checking the activitiy status of the `Tracer`.
#[derive(Debug)]
pub struct Tracer<T: core::Flow> {
    /// The receiver that used to activate/deactivate streams.
    active: watch::Receiver<bool>,
    description: Arc<Description>,
    mode: InnerMode<T>,
}

impl<T: core::Flow> Clone for Tracer<T> {
    fn clone(&self) -> Self {
        Self {
            active: self.active.clone(),
            description: self.description.clone(),
            mode: self.mode.clone(),
        }
    }
}

impl<T: core::Flow> Tracer<T> {
    /// Creates a new `Tracer`.
    pub fn new_tracer(state: T, path: Path, pull: Option<Duration>) -> Self {
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
        Self::new(path, inner_mode, mode)
    }

    /// Creates a new watched `Tracer`
    ///
    /// WARNING! Some control messages can be lost in case it the watcher
    /// is not subscribed yet when first messages received.
    // TODO: Fix the problem above ^
    pub fn new_watcher(state: T, path: Path) -> Self {
        let (tx, rx) = broadcast::channel(16);
        let mode = TracerMode::Watched {
            state,
            sender: tx.clone(),
        };
        let inner_mode = InnerMode::Watched {
            sender: Arc::new(tx),
        };
        Self::new(path, inner_mode, mode)
    }

    fn new(path: Path, inner_mode: InnerMode<T>, mode: TracerMode<T>) -> Self {
        let stream_type = T::stream_type();
        let info = format!("{} - {}", path, stream_type);
        let description = Description {
            path,
            info,
            stream_type,
        };
        // TODO: Remove this active watch channel?
        let (_active_tx, active_rx) = watch::channel(true);
        log::trace!("Creating Tracer with path: {}", description.path);
        let description = Arc::new(description);
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
            let ts = time_to_ts(opt_system_time);
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
                                T::apply(state, timed_event);
                            }
                            Err(err) => {
                                log::error!(
                                    "Can't lock the mutex to apply the changes of {}: {}",
                                    self.path(),
                                    err
                                );
                            }
                        },
                        InnerMode::Watched { .. } => {
                            log::error!(
                                "Sending is not supported in watched mode of {}",
                                self.path()
                            );
                        }
                    }
                }
                Err(err) => {
                    log::error!(
                        "Can't make a timestamp from provided system time of {}: {}",
                        self.path(),
                        err
                    );
                }
            }
        }
    }

    pub async fn subscribe(&mut self) -> Result<Watcher<T::Event>, Error> {
        match &mut self.mode {
            InnerMode::Watched { sender } => Ok(sender.subscribe()),
            InnerMode::Push { .. } => {
                log::error!("Can't receive state in push mode of {}", self.path(),);
                Err(Error::msg("Tracer::recv is not supported in push mode."))
            }
            InnerMode::Pull { .. } => {
                log::error!("Can't receive state in pull mode of {}", self.path(),);
                Err(Error::msg("Tracer::recv is not supported in pull mode."))
            }
        }
    }

    /*
    /// Receive a state.
    pub async fn recv(&mut self) -> Result<TimedEvent<T::Event>, Error> {
        match &mut self.mode {
            InnerMode::Watched { receiver: Some(receiver), .. } => {
                receiver.recv().await.map_err(Error::from)
            },
            InnerMode::Push { .. } => {
                log::error!("Can't receive state in push mode of {}", self.path(),);
                Err(Error::msg("Tracer::recv is not supported in push mode."))
            }
            InnerMode::Pull { .. } => {
                log::error!("Can't receive state in pull mode of {}", self.path(),);
                Err(Error::msg("Tracer::recv is not supported in pull mode."))
            }
        }
    }
    */
}

impl<T: core::Flow> Tracer<T> {
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

// TODO: How to avoid errors here?
pub(crate) fn time_to_ts(opt_system_time: Option<SystemTime>) -> Result<Timestamp, Error> {
    opt_system_time
        .unwrap_or_else(SystemTime::now)
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(Timestamp::from)
        .map_err(Error::from)
}

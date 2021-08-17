//! This module contains a generic `Tracer`'s methods.
use crate::actors::connector;
//use crate::actors::pool::{self, RillPoolTask};
use anyhow::Error;
use meio::Action;
use rill_protocol::flow::core::{self, ActionEnvelope, TimedEvent};
use rill_protocol::io::provider::{Description, Path, ProviderProtocol, Timestamp};
use rill_protocol::io::transport::Direction;
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;

#[derive(Debug)]
pub(crate) enum EventEnvelope<T: core::Flow> {
    Patch {
        direction: Option<Direction<ProviderProtocol>>,
        event: T::Event,
    },
}

impl<T: core::Flow> Action for EventEnvelope<T> {}

// TODO: Remove that aliases and use raw types receivers in recorders.
pub(crate) type DataSender<T> = mpsc::UnboundedSender<EventEnvelope<T>>;
pub(crate) type DataReceiver<T> = mpsc::UnboundedReceiver<EventEnvelope<T>>;

/// A sender for actions wrapped with an envelope.
pub type ControlSender<T> = mpsc::UnboundedSender<ActionEnvelope<T>>;
/// A receiver for actions.
pub type ControlReceiver<T> = mpsc::UnboundedReceiver<ActionEnvelope<T>>;

/// Creates a new control channel.
pub fn channel<T: core::Flow>() -> (ControlSender<T>, ControlReceiver<T>) {
    mpsc::unbounded_channel()
}

pub(crate) struct TracerOperator<T: core::Flow> {
    pub mode: TracerMode<T>,
    pub callback: Option<ControlSender<T>>,
}

pub(crate) enum TracerMode<T: core::Flow> {
    /// Real-time mode
    Push {
        state: T,
        receiver: Option<DataReceiver<T>>,
    },
    /// Pulling for intensive streams with high-load activities
    Pull {
        // TODO: Replace with `Arc` since data channel used
        // to detect Tracers's termination
        state: Weak<Mutex<T>>,
        interval: Duration,
    },
}

#[derive(Debug)]
enum InnerMode<T: core::Flow> {
    Push {
        // TODO: Add an optional buffer + flushing:
        // TODO: Also it's possible to add a special `AccumulatedDelta` subtype to `Flow`.
        // buffer: `Option<Vec<T>, usize>`,
        // if the `buffer` exists it does `autoflush`
        // or can be flushed manually by `tracer.flush()` call.
        sender: DataSender<T>,
    },
    Pull {
        state: Arc<Mutex<T>>,
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
        }
    }
}

/// The generic provider that forwards metrics to worker and keeps a flag
/// for checking the activitiy status of the `Tracer`.
#[derive(Debug)]
pub struct Tracer<T: core::Flow> {
    description: Arc<Description>,
    mode: InnerMode<T>,
}

impl<T: core::Flow> Clone for Tracer<T> {
    fn clone(&self) -> Self {
        Self {
            description: self.description.clone(),
            mode: self.mode.clone(),
        }
    }
}

// TODO: Not sure this is suitable for on-demand spawned recorders.
/// Both tracers are equal only if they use the same description.
/// That means they both have the same recorder/channel.
impl<T: core::Flow> PartialEq for Tracer<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.description, &other.description)
    }
}

impl<T: core::Flow> Eq for Tracer<T> {}

impl<T: core::Flow> Tracer<T> {
    /// Create a new `Tracer`
    pub fn new(
        state: T,
        path: Path,
        pull_interval: Option<Duration>,
        callback: Option<ControlSender<T>>,
    ) -> Self {
        if let Some(duration) = pull_interval {
            Self::new_pull(state, path, duration, callback)
        } else {
            Self::new_push(state, path, callback)
        }
    }

    /// Create a `Push` mode `Tracer`
    pub fn new_push(state: T, path: Path, callback: Option<ControlSender<T>>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let mode = TracerMode::Push {
            state,
            receiver: Some(rx),
        };
        let inner_mode = InnerMode::Push { sender: tx };
        Self::new_inner(path, inner_mode, mode, callback)
    }

    /// Create a `Pull` mode `Tracer`
    pub fn new_pull(
        state: T,
        path: Path,
        interval: Duration,
        callback: Option<ControlSender<T>>,
    ) -> Self {
        let state = Arc::new(Mutex::new(state));
        let mode = TracerMode::Pull {
            state: Arc::downgrade(&state),
            interval,
        };
        let inner_mode = InnerMode::Pull { state };
        Self::new_inner(path, inner_mode, mode, callback)
    }

    fn new_inner(
        path: Path,
        inner_mode: InnerMode<T>,
        mode: TracerMode<T>,
        callback: Option<ControlSender<T>>,
    ) -> Self {
        let operator = TracerOperator { mode, callback };
        let stream_type = T::stream_type();
        let info = format!("{} - {}", path, stream_type);
        let description = Description {
            path,
            info,
            stream_type,
        };
        log::trace!("Creating Tracer with path: {}", description.path);
        let description = Arc::new(description);
        let this = Tracer {
            description: description.clone(),
            mode: inner_mode,
        };
        if let Err(err) = connector::DISTRIBUTOR.register_tracer(description, operator) {
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

    /// Returns a reference to a `Description` of the `Tracer`.
    pub fn description(&self) -> &Description {
        &self.description
    }

    /// Ask recorder to resend a state in the `Pull` mode.
    pub fn flush(&self) {
        /* TODO: Use events
        match &self.mode {
            InnerMode::Pull { notifier, .. } => {
                notifier.notify_one();
            }
            InnerMode::Push { .. } => {
                // TODO: Implement buffering and flushing.
                log::error!("Buffering and flushing is not supported in `Push` mode yet");
            }
        }
        */
    }

    /// Send an event to a `Recorder`.
    pub fn send(&self, event: T::Event, direction: Option<Direction<ProviderProtocol>>) {
        match &self.mode {
            InnerMode::Push { sender, .. } => {
                let envelope = EventEnvelope::Patch { direction, event };
                // And will never send an event
                if let Err(err) = sender.send(envelope) {
                    log::error!("Can't transfer data to sender of {}: {}", self.path(), err);
                }
            }
            InnerMode::Pull { state, .. } => match state.lock() {
                // `direction` ignored always in the `Pull` mode
                Ok(ref mut state) => {
                    T::apply(state, event);
                }
                Err(err) => {
                    log::error!(
                        "Can't lock the mutex to apply the changes of {}: {}",
                        self.path(),
                        err
                    );
                }
            },
        }
    }

    /*
    /// Registers a callback to the flow.
    pub fn callback<F>(&mut self, func: F)
    where
        F: Fn(ActionEnvelope<T>) + Send + 'static,
    {
        let callback = Callback {
            tracer: self.clone(),
            callback: func,
        };
        if let Err(err) = pool::DISTRIBUTOR.spawn_task(callback) {
            log::error!(
                "Can't spawn a Callback. The worker can be terminated already: {}",
                err
            );
        }
    }
    */
}

/*
struct Callback<T: core::Flow, F> {
    tracer: Tracer<T>,
    callback: F,
}

#[async_trait]
impl<T, F> RillPoolTask for Callback<T, F>
where
    T: core::Flow,
    F: Fn(ActionEnvelope<T>) + Send + 'static,
{
    async fn routine(mut self) -> Result<(), Error> {
        let mut stream = self.tracer.subscribe()?;
        loop {
            let envelope = stream.recv().await?;
            (self.callback)(envelope)
        }
    }
}
*/

/// Wraps with timed event
pub fn timed<T>(event: T) -> Option<TimedEvent<T>> {
    time_to_ts(None)
        .map(move |timestamp| TimedEvent { timestamp, event })
        .ok()
}

/// Generates a `Timestamp` of converts `SystemTime` to it.
// TODO: How to avoid errors here?
pub fn time_to_ts(opt_system_time: Option<SystemTime>) -> Result<Timestamp, Error> {
    opt_system_time
        .unwrap_or_else(SystemTime::now)
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(Timestamp::from)
        .map_err(Error::from)
}

//! This module contains a generic `Tracer`'s methods.
use crate::actors::connector;
//use crate::actors::pool::{self, RillPoolTask};
use anyhow::Error;
use async_trait::async_trait;
use meio::Action;
use rill_protocol::flow::core::{self, Activity, TimedEvent};
use rill_protocol::io::provider::{Description, Path, ProviderProtocol, ProviderReqId, Timestamp};
use rill_protocol::io::transport::Direction;
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, SystemTime};
use tokio::sync::{mpsc, watch, Notify};

#[derive(Debug)]
pub(crate) struct EventEnvelope<T: core::Flow> {
    pub direction: Option<Direction<ProviderProtocol>>,
    pub event: T::Event,
}

impl<T: core::Flow> Action for EventEnvelope<T> {}

/*
struct EmptyCallback<T: core::Flow> {}

impl<T> ActionCallback<T> for EmptyCallback<T> {
    async fn handle_activity(
        &mut self,
        _origin: ProviderReqId,
        _activity: Activity<T>,
    ) -> Result<(), Error> {
        Ok(())
    }
}
*/

// TODO: Remove that aliases and use raw types receivers in recorders.
pub(crate) type DataSender<T> = mpsc::UnboundedSender<EventEnvelope<T>>;
pub(crate) type DataReceiver<T> = mpsc::UnboundedReceiver<EventEnvelope<T>>;

/// The callback that called on flow's incoming actions.
#[async_trait]
pub trait ActionCallback<T: core::Flow>: Send + Sync {
    /// When at least one connection exists.
    async fn awake(&mut self) {}

    /// When all clients disconnected.
    async fn suspend(&mut self) {}

    /// A method to handle an action.
    async fn handle_activity(
        &mut self,
        origin: ProviderReqId,
        activity: Activity<T>,
    ) -> Result<(), Error>;
}

/// Boxed callback.
pub type BoxedCallback<T> = Box<dyn ActionCallback<T>>;

pub(crate) struct TracerOperator<T: core::Flow> {
    pub mode: TracerMode<T>,
    pub callback: Option<BoxedCallback<T>>,
}

pub(crate) enum TracerMode<T: core::Flow> {
    /// Real-time mode
    Push {
        state: T,
        receiver: Option<DataReceiver<T>>,
        // For sending events to the `Tracer` instance
        //control_sender: Option<ControlSender<T>>,
    },
    /// Pulling for intensive streams with high-load activities
    Pull {
        state: Weak<Mutex<T>>,
        interval: Duration,
        notifier: Arc<Notify>,
        // TODO: It's also possible to support `control_sender`
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
        /// For sending notifications about important state changes
        /// IMPORTANT! No join with `Arc` of `state`, because
        /// state needs `Weak` to detect closing.
        notifier: Arc<Notify>,
    },
}

// TODO: Or require `Clone` for the `Flow` to derive this
impl<T: core::Flow> Clone for InnerMode<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Push { sender } => Self::Push {
                sender: sender.clone(),
            },
            Self::Pull { state, notifier } => Self::Pull {
                state: state.clone(),
                notifier: notifier.clone(),
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
        callback: Option<BoxedCallback<T>>,
    ) -> Self {
        if let Some(duration) = pull_interval {
            Self::new_pull(state, path, duration, callback)
        } else {
            Self::new_push(state, path, callback)
        }
    }

    /// Create a `Push` mode `Tracer`
    pub fn new_push(state: T, path: Path, callback: Option<BoxedCallback<T>>) -> Self {
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
        callback: Option<BoxedCallback<T>>,
    ) -> Self {
        let state = Arc::new(Mutex::new(state));
        let notifier = Arc::new(Notify::new());
        let mode = TracerMode::Pull {
            state: Arc::downgrade(&state),
            interval,
            notifier: notifier.clone(),
        };
        let inner_mode = InnerMode::Pull { state, notifier };
        Self::new_inner(path, inner_mode, mode, callback)
    }

    fn new_inner(
        path: Path,
        inner_mode: InnerMode<T>,
        mode: TracerMode<T>,
        callback: Option<BoxedCallback<T>>,
    ) -> Self {
        let operator = TracerOperator { mode, callback };
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
        if self.is_active() {
            match &self.mode {
                InnerMode::Pull { notifier, .. } => {
                    notifier.notify_one();
                }
                InnerMode::Push { .. } => {
                    // TODO: Implement buffering and flushing.
                    log::error!("Buffering and flushing is not supported in `Push` mode yet");
                }
            }
        }
    }

    /// Send an event to a `Recorder`.
    pub fn send(&self, event: T::Event, direction: Option<Direction<ProviderProtocol>>) {
        if self.is_active() {
            match &self.mode {
                InnerMode::Push { sender, .. } => {
                    let envelope = EventEnvelope { direction, event };
                    // And will never send an event
                    if let Err(err) = sender.send(envelope) {
                        log::error!("Can't transfer data to sender: {}", err);
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

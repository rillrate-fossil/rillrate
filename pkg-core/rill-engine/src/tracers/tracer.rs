//! This module contains a generic `Tracer`'s methods.
use crate::actors::connector;
use anyhow::Error;
use async_trait::async_trait;
use futures::Future;
use meio::Action;
use rill_protocol::flow::core::{ActionEnvelope, Flow, FlowMode};
use rill_protocol::io::provider::{Description, Path, ProviderProtocol};
use rill_protocol::io::transport::Direction;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug)]
pub(crate) struct EventEnvelope<T: Flow> {
    pub direction: Option<Direction<ProviderProtocol>>,
    pub event: T::Event,
}

pub(crate) enum ControlEvent<T> {
    Flush,
    AttachCallback { callback: BoxedCallback<T> },
    // AttachCallbackSender { sender: ActionSender<T> },
    DetachCallback,
}

impl<T: Flow> Action for EventEnvelope<T> {}

// TODO: Remove that aliases and use raw types receivers in recorders.
pub(crate) type DataSender<T> = mpsc::UnboundedSender<EventEnvelope<T>>;
pub(crate) type DataReceiver<T> = mpsc::UnboundedReceiver<EventEnvelope<T>>;

pub(crate) type ControlSender<T> = mpsc::UnboundedSender<ControlEvent<T>>;
pub(crate) type ControlReceiver<T> = mpsc::UnboundedReceiver<ControlEvent<T>>;

/// A sender for actions wrapped with an envelope.
pub type ActionSender<T> = mpsc::UnboundedSender<ActionEnvelope<T>>;
/// A receiver for actions.
pub type ActionReceiver<T> = mpsc::UnboundedReceiver<ActionEnvelope<T>>;

/// Creates a new control channel.
pub fn channel<T: Flow>() -> (ActionSender<T>, ActionReceiver<T>) {
    mpsc::unbounded_channel()
}

pub(crate) struct TracerOperator<T: Flow> {
    pub mode: TracerMode<T>,
    pub control_rx: Option<ControlReceiver<T>>,
}

pub(crate) enum TracerMode<T: Flow> {
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
        interval: Option<Duration>,
    },
}

#[derive(Debug)]
enum InnerMode<T: Flow> {
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
impl<T: Flow> Clone for InnerMode<T> {
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
pub struct Tracer<T: Flow> {
    description: Arc<Description>,
    control_tx: ControlSender<T>,
    mode: InnerMode<T>,
}

impl<T: Flow> Clone for Tracer<T> {
    fn clone(&self) -> Self {
        Self {
            description: self.description.clone(),
            control_tx: self.control_tx.clone(),
            mode: self.mode.clone(),
        }
    }
}

// TODO: Not sure this is suitable for on-demand spawned recorders.
/// Both tracers are equal only if they use the same description.
/// That means they both have the same recorder/channel.
impl<T: Flow> PartialEq for Tracer<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.description, &other.description)
    }
}

impl<T: Flow> Eq for Tracer<T> {}

impl<T: Flow> Tracer<T> {
    /// Create a new `Tracer`
    pub fn new(state: T, path: Path, mode: FlowMode) -> Self {
        match mode {
            FlowMode::Realtime => Self::new_push(state, path),
            FlowMode::Throttle { ms } => {
                Self::new_pull(state, path, Some(Duration::from_millis(ms)))
            }
            FlowMode::FlushOnly => Self::new_pull(state, path, None),
        }
    }

    /// Create a `Push` mode `Tracer`
    pub fn new_push(state: T, path: Path) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let mode = TracerMode::Push {
            state,
            receiver: Some(rx),
        };
        let inner_mode = InnerMode::Push { sender: tx };
        Self::new_inner(path, inner_mode, mode)
    }

    /// Create a `Pull` mode `Tracer`
    pub fn new_pull(state: T, path: Path, interval: Option<Duration>) -> Self {
        let state = Arc::new(Mutex::new(state));
        let mode = TracerMode::Pull {
            state: Arc::downgrade(&state),
            interval,
        };
        let inner_mode = InnerMode::Pull { state };
        Self::new_inner(path, inner_mode, mode)
    }

    fn new_inner(path: Path, inner_mode: InnerMode<T>, mode: TracerMode<T>) -> Self {
        let (control_tx, control_rx) = mpsc::unbounded_channel();
        let operator = TracerOperator {
            mode,
            control_rx: Some(control_rx),
        };
        let stream_type = T::stream_type();
        let description = Description { path, stream_type };
        log::trace!("Creating Tracer with path: {}", description.path);
        let description = Arc::new(description);
        let this = Tracer {
            description: description.clone(),
            control_tx,
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

    /// Send an event to a `Recorder`.
    // TODO: Consider using explicit direction value. What sould Broadcast be?
    pub fn send(&self, event: T::Event, direction: Option<Direction<ProviderProtocol>>) {
        match &self.mode {
            InnerMode::Push { sender, .. } => {
                let envelope = EventEnvelope { direction, event };
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

    /// Ask recorder to resend a state in the `Pull` mode.
    pub fn flush(&self) {
        let event = ControlEvent::Flush;
        if let Err(err) = self.control_tx.send(event) {
            log::error!("Can't send a flush event to {}: {}", self.path(), err);
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

    /// Assign a sync callback
    pub fn sync_callback<F>(&self, callback: F)
    where
        F: Fn(ActionEnvelope<T>) -> Result<(), Error>,
        F: Send + Sync + 'static,
    {
        let sync_callback = SyncCallback::new(callback);
        let callback = Box::new(sync_callback);
        let event = ControlEvent::AttachCallback { callback };
        if let Err(err) = self.control_tx.send(event) {
            log::error!("Can't attach the callback from {}: {}", self.path(), err);
        }
    }

    /// Assign an async callback
    pub fn async_callback<F, Fut>(&self, callback: F)
    where
        F: Fn(ActionEnvelope<T>) -> Fut,
        F: Send + Sync + 'static,
        Fut: Future<Output = Result<(), Error>>,
        Fut: Send + 'static,
    {
        let async_callback = AsyncCallback::new(callback);
        let callback = Box::new(async_callback);
        let event = ControlEvent::AttachCallback { callback };
        if let Err(err) = self.control_tx.send(event) {
            log::error!("Can't attach the callback from {}: {}", self.path(), err);
        }
    }

    /// Removes the callback
    pub fn detach_callback(&self) {
        let event = ControlEvent::DetachCallback;
        if let Err(err) = self.control_tx.send(event) {
            log::error!("Can't detach the callback from {}: {}", self.path(), err);
        }
    }
}

/// The callback that called on flow's incoming actions.
#[async_trait]
pub trait ActionCallback<T: Flow>: Send + 'static {
    /*
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
    */

    // TODO: Make it sync
    /// A method to handle an action.
    async fn handle_activity(&mut self, envelope: ActionEnvelope<T>) -> Result<(), Error>;
}

/// Boxed callback.
pub type BoxedCallback<T> = Box<dyn ActionCallback<T>>;

/// Sync callback.
struct SyncCallback<F> {
    func: Arc<F>,
}

impl<F> SyncCallback<F> {
    fn new(func: F) -> Self {
        Self {
            func: Arc::new(func),
        }
    }
}

#[async_trait]
impl<T, F> ActionCallback<T> for SyncCallback<F>
where
    T: Flow,
    F: Fn(ActionEnvelope<T>) -> Result<(), Error>,
    F: Send + Sync + 'static,
{
    async fn handle_activity(&mut self, envelope: ActionEnvelope<T>) -> Result<(), Error> {
        let func = self.func.clone();
        tokio::task::spawn_blocking(move || func(envelope))
            .await
            .map_err(Error::from)
            .and_then(std::convert::identity)
    }
}

use std::marker::PhantomData;

struct AsyncCallback<F, Fut> {
    func: F,
    fut: PhantomData<Fut>,
}

impl<F, Fut> AsyncCallback<F, Fut> {
    fn new(func: F) -> Self {
        Self {
            func: func,
            fut: PhantomData,
        }
    }
}

#[async_trait]
impl<T, F, Fut> ActionCallback<T> for AsyncCallback<F, Fut>
where
    T: Flow,
    F: Fn(ActionEnvelope<T>) -> Fut,
    F: Send + Sync + 'static,
    Fut: Future<Output = Result<(), Error>>,
    Fut: Send + 'static,
{
    async fn handle_activity(&mut self, envelope: ActionEnvelope<T>) -> Result<(), Error> {
        (self.func)(envelope).await
    }
}

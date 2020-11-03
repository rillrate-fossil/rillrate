use crate::protocol::{Path, RillData, StreamId};
use futures::channel::mpsc;
use meio::Action;
use once_cell::sync::OnceCell;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Keeps `StreamId` and implements `Action`.
#[derive(Debug)]
pub struct DataEnvelope {
    pub stream_id: StreamId,
    pub data: RillData,
}

impl Action for DataEnvelope {}

pub type DataSender = mpsc::UnboundedSender<DataEnvelope>;
pub type DataReceiver = mpsc::UnboundedReceiver<DataEnvelope>;

#[derive(Debug)]
pub struct Provider {
    stream_id: StreamId,
    active: AtomicBool,
    sender: DataSender,
}

impl Provider {
    pub fn create(stream_id: StreamId) -> (DataReceiver, Self) {
        let (tx, rx) = mpsc::unbounded();
        let this = Self {
            stream_id,
            active: AtomicBool::new(false),
            sender: tx,
        };
        (rx, this)
    }

    pub fn switch(&self, active: bool) {
        self.active.store(active, Ordering::Relaxed);
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    fn send(&self, data: RillData) {
        let envelope = DataEnvelope {
            stream_id: self.stream_id,
            data,
        };
        self.sender.unbounded_send(envelope).ok();
    }
}

pub struct StaticJoint {
    module: &'static str,
    provider: OnceCell<Provider>,
}

impl Deref for StaticJoint {
    type Target = Provider;

    fn deref(&self) -> &Provider {
        self.provider.get().expect("not registered StaticJoint")
    }
}

impl StaticJoint {
    pub const fn new(module: &'static str) -> Self {
        Self {
            module,
            provider: OnceCell::new(),
        }
    }

    pub fn register(&'static self) {
        let state = crate::RILL_STATE.get().expect("rill not installed!");
        let stream_id = state.next();
        // IMPORTANT: Initialize `Provider` here to create the channel before it
        // will be used by the user.
        let (rx, provider) = Provider::create(stream_id);
        self.provider
            .set(provider)
            .expect("provider already initialized");
        let event = ControlEvent::RegisterStaticStream { provider: self, rx };
        state.send(event);
    }

    pub fn stream_id(&self) -> StreamId {
        if let Some(provider) = self.provider.get() {
            provider.stream_id.clone()
        } else {
            panic!("uninitialized stream");
        }
    }

    pub fn log(&self, message: String) {
        if let Some(provider) = self.provider.get() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let data = RillData::LogRecord {
                timestamp: now as i64, //TODO: Change to u128 instead?
                message,
            };
            provider.send(data);
        }
    }

    pub fn path(&self) -> Path {
        self.module
            .split("::")
            .map(String::from)
            .collect::<Vec<_>>()
            .into()
    }
}

pub struct DynamicJoint {
    module: String,
    active: AtomicBool,
    provider: Provider,
}

impl DynamicJoint {
    pub fn create_and_register(module: &str) -> Arc<Self> {
        let state = crate::RILL_STATE.get().expect("rill not installed!");
        let stream_id = state.next();
        let (rx, provider) = Provider::create(stream_id);
        let this = Self {
            module: module.to_string(),
            active: AtomicBool::new(false),
            provider,
        };
        let joint = Arc::new(this);
        // Registering
        let event = ControlEvent::RegisterDynamicStream {
            provider: joint.clone(),
            rx,
        };
        state.send(event);
        joint
    }

    // TODO: DRY
    pub fn stream_id(&self) -> StreamId {
        self.provider.stream_id.clone()
    }

    // TODO: DRY
    pub fn switch(&self, active: bool) {
        self.active.store(active, Ordering::Relaxed);
    }

    // TODO: DRY
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    // TODO: DRY
    pub fn path(&self) -> Path {
        self.module
            .split("::")
            .map(String::from)
            .collect::<Vec<_>>()
            .into()
    }
}

pub enum ControlEvent {
    RegisterStaticStream {
        provider: &'static StaticJoint,
        rx: DataReceiver,
    },
    RegisterDynamicStream {
        provider: Arc<DynamicJoint>,
        rx: DataReceiver,
    },
}

impl Action for ControlEvent {}

pub type ControlSender = mpsc::UnboundedSender<ControlEvent>;
pub type ControlReceiver = mpsc::UnboundedReceiver<ControlEvent>;

pub struct RillState {
    sender: ControlSender,
    stream_id_counter: AtomicUsize,
}

impl RillState {
    pub fn create() -> (ControlReceiver, Self) {
        let (tx, rx) = mpsc::unbounded();
        let this = Self {
            sender: tx,
            stream_id_counter: AtomicUsize::new(0),
        };
        (rx, this)
    }

    fn next(&self) -> StreamId {
        let id = self.stream_id_counter.fetch_add(1, Ordering::Relaxed);
        StreamId(id as u64)
    }

    fn send(&self, event: ControlEvent) {
        self.sender
            .unbounded_send(event)
            .expect("rill actors not started");
    }
}

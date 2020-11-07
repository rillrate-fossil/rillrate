use crate::protocol::{EntryId, Path, RillData};
use derive_more::From;
use futures::channel::mpsc;
use meio::Action;
use once_cell::sync::OnceCell;
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Keeps `EntryId` and implements `Action`.
#[derive(Debug)]
pub struct DataEnvelope {
    pub entry_id: EntryId,
    pub data: RillData,
}

impl Action for DataEnvelope {}

pub type DataSender = mpsc::UnboundedSender<DataEnvelope>;
pub type DataReceiver = mpsc::UnboundedReceiver<DataEnvelope>;

#[derive(Debug)]
pub struct Provider {
    entry_id: EntryId,
    active: AtomicBool,
    sender: DataSender,
}

impl Provider {
    pub fn create(entry_id: EntryId) -> (DataReceiver, Self) {
        let (tx, rx) = mpsc::unbounded();
        let this = Self {
            entry_id,
            active: AtomicBool::new(false),
            sender: tx,
        };
        (rx, this)
    }

    pub fn entry_id(&self) -> &EntryId {
        &self.entry_id
    }

    pub fn switch(&self, active: bool) {
        self.active.store(active, Ordering::Relaxed);
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    fn send(&self, data: RillData) {
        let envelope = DataEnvelope {
            // TODO: Use `DirectId` here, or nothing.
            entry_id: self.entry_id.clone(),
            data,
        };
        self.sender.unbounded_send(envelope).ok();
    }
}

pub trait Joint: Deref<Target = Provider> + Sync + Send {
    fn module(&self) -> &str;
}

impl dyn Joint {
    pub fn path(&self) -> Path {
        self.module()
            .split("::")
            .map(EntryId::from)
            .collect::<Vec<_>>()
            .into()
    }
}

#[derive(Clone, From)]
pub struct StaticJointWrapper {
    inner: &'static StaticJoint,
}

impl Deref for StaticJointWrapper {
    type Target = Provider;

    fn deref(&self) -> &Provider {
        self.inner
            .provider
            .get()
            .expect("not registered StaticJoint")
    }
}

/// Statically embedded provider, recommended for languages that supports
/// `const` expressions.
pub struct StaticJoint {
    module: &'static str,
    provider: OnceCell<Provider>,
}

impl Joint for StaticJointWrapper {
    fn module(&self) -> &str {
        self.inner.module
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
        let entry_id = EntryId::from(self.module);
        // IMPORTANT: Initialize `Provider` here to create the channel before it
        // will be used by the user.
        let (rx, provider) = Provider::create(entry_id);
        self.provider
            .set(provider)
            .expect("provider already initialized");
        let wrapper = StaticJointWrapper { inner: self };
        let joint: Box<dyn Joint> = Box::new(wrapper);
        let event = ControlEvent::RegisterJoint { joint, rx };
        state.send(event);
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
}

/// Provides data from a dynamic environment.
#[derive(Clone)]
pub struct DynamicJoint {
    inner: Arc<DynamicJointInner>,
}

impl Deref for DynamicJoint {
    type Target = Provider;

    fn deref(&self) -> &Provider {
        &self.inner.provider
    }
}

impl Joint for DynamicJoint {
    fn module(&self) -> &str {
        &self.inner.module
    }
}

impl DynamicJoint {
    pub fn create_and_register(module: &str) -> Self {
        let state = crate::RILL_STATE.get().expect("rill not installed!");
        let entry_id = EntryId::from(module);
        let (rx, provider) = Provider::create(entry_id);
        let inner = DynamicJointInner {
            module: module.to_string(),
            provider,
        };
        let joint = Self {
            inner: Arc::new(inner),
        };
        // Registering
        let boxed_joint: Box<dyn Joint> = Box::new(joint.clone());
        let event = ControlEvent::RegisterJoint {
            joint: boxed_joint,
            rx,
        };
        state.send(event);
        joint
    }

    pub fn log(&self, message: String) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let data = RillData::LogRecord {
            timestamp: now as i64, //TODO: Change to u128 instead?
            message,
        };
        self.send(data);
    }
}

struct DynamicJointInner {
    module: String,
    provider: Provider,
}

pub enum ControlEvent {
    // TODO: Use the single `RegisterAllJoints` event with no `Completed` variant.
    RegisterJoint {
        joint: Box<dyn Joint>,
        rx: DataReceiver,
    },
}

impl Action for ControlEvent {}

pub type ControlSender = mpsc::UnboundedSender<ControlEvent>;
pub type ControlReceiver = mpsc::UnboundedReceiver<ControlEvent>;

pub struct RillState {
    sender: ControlSender,
}

impl RillState {
    pub fn create() -> (ControlReceiver, Self) {
        let (tx, rx) = mpsc::unbounded();
        let this = Self { sender: tx };
        (rx, this)
    }

    fn send(&self, event: ControlEvent) {
        self.sender
            .unbounded_send(event)
            .expect("rill actors not started");
    }
}

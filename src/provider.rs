use crate::protocol::{Path, RillData, StreamId};
use futures::channel::mpsc;
use meio::Action;
use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};

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
    sender: DataSender,
}

impl Provider {
    pub fn create(stream_id: StreamId) -> (DataReceiver, Self) {
        let (tx, rx) = mpsc::unbounded();
        let this = Self {
            stream_id,
            sender: tx,
        };
        (rx, this)
    }

    fn send(&self, data: RillData) {
        let envelope = DataEnvelope {
            stream_id: self.stream_id,
            data,
        };
        self.sender.unbounded_send(envelope).ok();
    }
}

pub struct ProviderCell {
    module: &'static str,
    active: AtomicBool,
    provider: OnceCell<Provider>,
}

impl ProviderCell {
    pub const fn new(module: &'static str) -> Self {
        Self {
            module,
            active: AtomicBool::new(false),
            provider: OnceCell::new(),
        }
    }

    pub fn init(&self, stream_id: StreamId) -> DataReceiver {
        let (rx, provider) = Provider::create(stream_id);
        self.provider
            .set(provider)
            .expect("provider already initialized");
        rx
    }

    pub fn stream_id(&self) -> StreamId {
        if let Some(provider) = self.provider.get() {
            provider.stream_id.clone()
        } else {
            panic!("uninitialized stream");
        }
    }

    pub fn switch(&self, active: bool) {
        self.active.store(active, Ordering::Relaxed);
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    pub fn log(&self, message: String) {
        if self.is_active() {
            // TODO: Render message here! Only when provider is available.
            if let Some(provider) = self.provider.get() {
                let data = RillData::LogRecord {
                    timestamp: 0,
                    message,
                };
                provider.send(data);
            }
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

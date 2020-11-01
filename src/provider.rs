use crate::protocol::Path;
use futures::channel::mpsc;
use meio::Action;
use once_cell::sync::OnceCell;

pub enum Data {
    LogRecord { message: String },
}

impl Action for Data {}

pub type DataSender = mpsc::UnboundedSender<Data>;
pub type DataReceiver = mpsc::UnboundedReceiver<Data>;

/// `Connector` creates a provider and puts it into an `OnceCell` to start the stream.
pub struct Provider {
    sender: DataSender,
}

impl Provider {
    pub fn create() -> (DataReceiver, Self) {
        let (tx, rx) = mpsc::unbounded();
        let this = Self { sender: tx };
        (rx, this)
    }

    fn send(&self, data: Data) {
        self.sender.unbounded_send(data).ok();
    }
}

pub struct ProviderCell {
    module: &'static str,
    provider: OnceCell<Provider>,
}

impl ProviderCell {
    pub const fn new(module: &'static str) -> Self {
        Self {
            module,
            provider: OnceCell::new(),
        }
    }

    pub fn switch_on(&self) -> DataReceiver {
        let (rx, provider) = Provider::create();
        self.provider.set(provider);
        rx
    }

    pub fn switch_off(&mut self) {
        self.provider.take();
    }

    pub fn is_active(&self) -> bool {
        self.provider.get().is_some()
    }

    pub fn log(&self, message: String) {
        if let Some(provider) = self.provider.get() {
            // TODO: Render message here! Only when provider is available.
            let data = Data::LogRecord { message };
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

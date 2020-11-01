use crate::protocol::Path;
use futures::channel::mpsc;
use once_cell::sync::OnceCell;

pub type Data = String;

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

    fn send(&self, data: String) {
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

    pub fn init(&self) -> DataReceiver {
        let (rx, provider) = Provider::create();
        self.provider.set(provider);
        rx
    }

    pub fn log(&self, data: String) {
        if let Some(provider) = self.provider.get() {
            // TODO: Render data here! Only when provider is available.
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

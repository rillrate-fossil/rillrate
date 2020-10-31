use futures::channel::mpsc;
use once_cell::sync::OnceCell;

pub type Data = String;

/// `Connector` creates a provider and puts it into an `OnceCell` to start the stream.
pub struct Provider {
    sender: mpsc::UnboundedSender<Data>,
}

impl Provider {
    pub fn create() -> (mpsc::UnboundedReceiver<Data>, Self) {
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

    pub fn log(&self, data: String) {
        if let Some(provider) = self.provider.get() {
            provider.send(data);
        }
    }
}

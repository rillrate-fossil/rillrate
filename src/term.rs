use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

// TODO: Hide fields and give methods...
pub struct Receiver {
    pub notifier_rx: oneshot::Receiver<()>,
    pub blocker: Arc<Mutex<()>>,
}

// TODO: Hide fields and give methods...
pub struct Sender {
    pub notifier_tx: oneshot::Sender<()>,
    pub blocker: Arc<Mutex<()>>,
}

pub fn channel() -> (Sender, Receiver) {
    let (tx, rx) = oneshot::channel();
    let blocker = Arc::new(Mutex::new(()));
    let sender = Sender {
        notifier_tx: tx,
        blocker: blocker.clone(),
    };
    let receiver = Receiver {
        notifier_rx: rx,
        blocker,
    };
    (sender, receiver)
}

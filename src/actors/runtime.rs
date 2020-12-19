use super::supervisor::RillSupervisor;
use crate::protocol::EntryId;
use crate::state::ControlReceiver;
use anyhow::Error;
use meio::prelude::System;

#[tokio::main]
pub(crate) async fn entrypoint(
    entry_id: EntryId,
    rx: ControlReceiver,
    term_rx: term::Receiver,
) -> Result<(), Error> {
    let blocker = term_rx
        .blocker
        .lock()
        .map_err(|_| Error::msg("can't take termination blocker"))?;
    let mut handle = System::spawn(RillSupervisor::new(entry_id, rx));
    term_rx.notifier_rx.await?;
    System::interrupt(&mut handle)?;
    handle.join().await;
    drop(blocker);
    Ok(())
}

pub mod term {
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
}

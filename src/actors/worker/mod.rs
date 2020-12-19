mod actor;
pub use actor::RillWorker;

use crate::protocol::EntryId;
use crate::state::ControlReceiver;
use crate::term;
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
    let mut handle = System::spawn(RillWorker::new(entry_id));
    handle.attach(rx);
    term_rx.notifier_rx.await?;
    System::interrupt(&mut handle)?;
    handle.join().await;
    drop(blocker);
    Ok(())
}

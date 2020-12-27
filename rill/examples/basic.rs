use anyhow::Error;
use rill::prelude::{CounterProvider, Rill};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    let _rill = Rill::install("basic-example")?;
    let counter = CounterProvider::new("counter".parse()?);

    // TODO: DRY it
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    while running.load(Ordering::SeqCst) {
        counter.inc(1.0, None);
        thread::sleep(Duration::from_millis(500));
    }

    Ok(())
}

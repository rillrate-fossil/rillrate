use anyhow::Error;
use rill::prelude::{CounterProvider, Rill};
use rill_export::RillExport;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    let _rill_export = RillExport::start()?;
    let _rill = Rill::install("example")?;

    // TODO: DRY it
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    let counter = CounterProvider::new("my.counter".parse()?);
    while running.load(Ordering::SeqCst) {
        counter.inc(1.0, None);
        thread::sleep(Duration::from_millis(500));
    }

    Ok(())
}

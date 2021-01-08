use anyhow::Error;
use rill::prelude::{CounterProvider, GaugeProvider, LogProvider, Rill};
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
    let gauge = GaugeProvider::new("my.gauge".parse()?);
    let fast_gauge = GaugeProvider::new("my.gauge.fast".parse()?);
    let random_gauge = GaugeProvider::new("my.gauge.random".parse()?);
    let logger = LogProvider::new("my.direct.logs.trace".parse()?);
    while running.load(Ordering::SeqCst) {
        for x in 0..3 {
            fast_gauge.set(x as f64, None);
            random_gauge.set(rand::random(), None);
            thread::sleep(Duration::from_millis(100));
        }
        gauge.set(1.0, None);
        for x in 0..7 {
            fast_gauge.set(x as f64, None);
            random_gauge.set(rand::random(), None);
            thread::sleep(Duration::from_millis(100));
        }
        gauge.set(10.0, None);
        counter.inc(1.0, None);
        logger.log(format!("okay :)"), None);
    }

    Ok(())
}

use anyhow::Error;
use rillrate::{CounterTracer, GaugeTracer, LogTracer, RillRate};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    let _rillrate = RillRate::from_env("all-example")?;

    // TODO: DRY it
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    {
        let counter_one = CounterTracer::new("my.counter.one".parse()?);
        let counter_two = CounterTracer::new("my.counter.two".parse()?);
        let gauge = GaugeTracer::new("my.gauge".parse()?);
        let fast_gauge = GaugeTracer::new("my.gauge.fast".parse()?);
        let random_gauge = GaugeTracer::new("my.gauge.random".parse()?);
        let logger = LogTracer::new("my.direct.logs.trace".parse()?);
        let fast_logger = LogTracer::new("my.direct.logs.fast".parse()?);
        let mut counter = 0;
        while running.load(Ordering::SeqCst) {
            counter += 1;
            for x in 0..3 {
                counter_two.inc(1.0, None);
                fast_gauge.set(x as f64, None);
                random_gauge.set(rand::random(), None);
                thread::sleep(Duration::from_millis(100));
                fast_logger.log(format!("first loop - {}/{}", counter, x), None);
            }
            gauge.set(1.0, None);
            for x in 0..7 {
                counter_two.inc(1.0, None);
                fast_gauge.set(x as f64, None);
                random_gauge.set(rand::random(), None);
                thread::sleep(Duration::from_millis(100));
                fast_logger.log(format!("second loop - {}/{}", counter, x), None);
            }
            gauge.set(10.0, None);
            counter_two.inc(1.0, None);
            counter_one.inc(1.0, None);
            logger.log(format!("okay :) - {}", counter), None);
        }
    }

    Ok(())
}

use anyhow::Error;
use rillrate::{Counter, Dict, Gauge, Logger, RillRate};
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
        // This gauge used to check that the Prometheus exporter can read
        // data from a snapshot event if the data will never updated again.
        let instances = Gauge::create("my.gauge.instances")?;
        instances.inc(1.0);

        let counter_one = Counter::create("my.counter.one")?;
        let counter_two = Counter::create("my.counter.two")?;
        let gauge = Gauge::create("my.gauge")?;
        let fast_gauge = Gauge::create("my.gauge.fast")?;
        let random_gauge = Gauge::create("my.gauge.random")?;
        let logger = Logger::create("my.direct.logs.trace")?;
        let fast_logger = Logger::create("my.direct.logs.fast")?;

        let mt_gauge = Gauge::create("my.gauge.multithread")?;

        let my_dict = Dict::create("my.dict.key-value")?;

        for i in 1..=5 {
            let mt_gauge_cloned = mt_gauge.clone();
            let running_cloned = running.clone();
            let tname = format!("thread-{}", i);
            thread::Builder::new().name(tname).spawn(move || {
                while running_cloned.load(Ordering::SeqCst) {
                    mt_gauge_cloned.set(i as f64);
                    thread::sleep(Duration::from_secs(1));
                }
            })?;
        }

        let mut counter = 0;
        while running.load(Ordering::SeqCst) {
            mt_gauge.set(0.0);
            counter += 1;
            my_dict.set("state", "step 1");
            for x in 0..3 {
                counter_two.inc(1.0);
                fast_gauge.set(x as f64);
                random_gauge.set(rand::random());
                thread::sleep(Duration::from_millis(100));
                fast_logger.log(format!("first loop - {}/{}", counter, x));
            }
            gauge.set(1.0);
            my_dict.set("state", "step 2");
            for x in 0..7 {
                counter_two.inc(1.0);
                fast_gauge.set(x as f64);
                random_gauge.set(rand::random());
                thread::sleep(Duration::from_millis(100));
                fast_logger.log(format!("second loop - {}/{}", counter, x));
            }
            my_dict.set("state", "last");
            gauge.set(10.0);
            counter_two.inc(1.0);
            counter_one.inc(1.0);
            logger.log(format!("okay :) - {}", counter));
            my_dict.set("iteration", counter);
        }
    }

    Ok(())
}

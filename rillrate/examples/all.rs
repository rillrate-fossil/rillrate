use anyhow::Error;
use rand::Rng;
use rillrate::{Col, Counter, Dict, Gauge, Histogram, Logger, Pulse, RillRate, Row, Table};
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
        // This pulse used to check that the Prometheus exporter can read
        // data from a snapshot event if the data will never updated again.
        let instances = Pulse::create("my.pulse.instances", None)?;
        instances.inc(1.0);

        let counter_one = Counter::create("my.counter.one")?;
        let counter_two = Counter::create("my.counter.two")?;
        let pulse = Pulse::create("my.pulse", None)?;
        let fast_pulse = Pulse::create("my.pulse.fast", None)?;
        let random_pulse = Pulse::create("my.pulse.random", None)?;
        let logger = Logger::create("my.direct.logs.trace")?;
        let fast_logger = Logger::create("my.direct.logs.fast")?;

        let mt_pulse = Pulse::create("my.pulse.multithread", None)?;

        let my_dict = Dict::create("my.dict.key-value")?;

        // TODO: Add and use `ToAlias` trait
        let my_table = Table::create("my.table.one", vec![(Col(0), "Thread"), (Col(1), "State")])?;

        for i in 1..=5 {
            let tbl = my_table.clone();
            let tname = format!("thread-{}", i);
            tbl.add_row(Row(i));
            tbl.set_cell(Row(i), Col(0), &tname);
            let mt_pulse_cloned = mt_pulse.clone();
            let running_cloned = running.clone();
            thread::Builder::new().name(tname).spawn(move || {
                while running_cloned.load(Ordering::SeqCst) {
                    mt_pulse_cloned.set(i as f64);
                    tbl.set_cell(Row(i), Col(1), "wait 1");
                    thread::sleep(Duration::from_millis(500));
                    tbl.set_cell(Row(i), Col(1), "wait 2");
                    thread::sleep(Duration::from_millis(500));
                }
            })?;
        }

        let my_gauge = Gauge::create("my.gauge", 0.0, 100.0)?;
        let my_hist = Histogram::create(
            "my.histogram",
            &[10.0, 50.0, 100.0, 200.0, 500.0, 1_000.0, 5_000.0],
        )?;

        let mut counter = 0;
        while running.load(Ordering::SeqCst) {
            mt_pulse.set(0.0);
            counter += 1;
            my_dict.set("state", "step 1");
            my_gauge.set(rand::thread_rng().gen_range(0.0..100.0));
            my_hist.add(150.0);
            for x in 0..3 {
                counter_two.inc(1.0);
                fast_pulse.set(x as f64);
                random_pulse.set(rand::random());
                thread::sleep(Duration::from_millis(100));
                fast_logger.log(format!("first loop - {}/{}", counter, x));
                my_hist.add(30.0);
            }
            pulse.set(1.0);
            my_dict.set("state", "step 2");
            for x in 0..7 {
                counter_two.inc(1.0);
                fast_pulse.set(x as f64);
                random_pulse.set(rand::random());
                thread::sleep(Duration::from_millis(100));
                fast_logger.log(format!("second loop - {}/{}", counter, x));
                my_hist.add(100.0);
            }
            my_dict.set("state", "last");
            pulse.set(10.0);
            counter_two.inc(1.0);
            counter_one.inc(1.0);
            logger.log(format!("okay :) - {}", counter));
            my_dict.set("iteration", counter);
        }
    }

    Ok(())
}

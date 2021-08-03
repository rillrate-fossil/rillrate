use anyhow::Error;
use std::thread;
use std::time::Duration;

pub fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    let handle = rillrate::start();
    let counter_1 = rillrate::CounterTracer::new("Demo Counter 1".parse()?, None);
    let counter_2 = rillrate::CounterTracer::new("Demo Counter 2".parse()?, None);
    let counter_3 = rillrate::CounterTracer::new("Demo Counter 3".parse()?, None);
    for _ in 1..=300 {
        counter_1.inc(1);
        counter_2.inc(10);
        counter_3.inc(100);
        thread::sleep(Duration::from_secs(1));
    }
    /*
    for _ in 1..100 {
        drop(tracer);
        thread::sleep(Duration::from_secs(3));
    }
    */
    thread::sleep(Duration::from_secs(1_000_000));
    Ok(())
}

use anyhow::Error;
use std::thread;
use std::time::Duration;

pub fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    let handle = rillrate::start();
    let tracer = rillrate::CounterTracer::new("Demo Counter".parse()?, None);
    for _ in 1..100 {
        thread::sleep(Duration::from_secs(1));
        tracer.inc(1);
    }
    thread::sleep(Duration::from_secs(1_000_000));
    Ok(())
}

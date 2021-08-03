use anyhow::Error;
use std::thread;
use std::time::Duration;

pub fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    let _handle = rillrate::start();
    let counter_1 = rillrate::CounterStatTracer::new("counter-1", true);
    let counter_2 = rillrate::CounterStatTracer::new("counter-2", true);
    let counter_3 = rillrate::CounterStatTracer::new("counter-3", true);
    let pulse_1 = rillrate::PulseFrameTracer::new("pulse-1");
    for x in 1..=300 {
        counter_1.inc(1);
        counter_2.inc(10);
        counter_3.inc(100);
        pulse_1.add(x as f32);
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

use anyhow::Error;
use std::thread;
use std::time::Duration;

const GROUP_1: &str = "group-1";
const GROUP_2: &str = "group-2";

pub fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    let _handle = rillrate::start();
    let counter_1 = rillrate::CounterStatTracer::new(GROUP_1.into(), "counter-1".into(), true);
    let counter_2 = rillrate::CounterStatTracer::new(GROUP_1.into(), "counter-2".into(), true);
    let counter_3 = rillrate::CounterStatTracer::new(GROUP_1.into(), "counter-3".into(), true);
    let pulse_1 = rillrate::PulseFrameTracer::new(GROUP_2.into(), "pulse-1".into());
    loop {
        for x in 1..=10 {
            counter_1.inc(1);
            counter_2.inc(10);
            counter_3.inc(100);
            pulse_1.add(x as f32);
            thread::sleep(Duration::from_secs(1));
        }
        let pulse_2 = rillrate::PulseFrameTracer::new(GROUP_2.into(), "pulse-2".into());
        for x in 1..=50 {
            counter_1.inc(1);
            counter_2.inc(10);
            counter_3.inc(100);
            pulse_1.add(x as f32);
            pulse_2.add(x as f32);
            thread::sleep(Duration::from_millis(500 - x * 10));
        }
        thread::sleep(Duration::from_secs(1));
    }
}

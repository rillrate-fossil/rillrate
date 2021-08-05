use anyhow::Error;
use rillrate::*;
use std::thread;
use std::time::Duration;

const PACKAGE_1: &str = "package-1";
const DASHBOARD_1: &str = "dashboard-1";
const DASHBOARD_2: &str = "dashboard-2";

const GROUP_1: &str = "group-1";
const GROUP_2: &str = "group-2";

pub fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    let _handle = rillrate::start();
    let counter_1 =
        CounterStatTracer::new([PACKAGE_1, DASHBOARD_1, GROUP_1, "counter-1"].into(), true);
    let counter_2 =
        CounterStatTracer::new([PACKAGE_1, DASHBOARD_1, GROUP_1, "counter-2"].into(), true);
    let counter_3 =
        CounterStatTracer::new([PACKAGE_1, DASHBOARD_1, GROUP_2, "counter-3"].into(), true);
    let pulse_1 = PulseFrameTracer::new([PACKAGE_1, DASHBOARD_2, GROUP_1, "pulse-1"].into());
    let board_1 = BoardListTracer::new([PACKAGE_1, DASHBOARD_2, GROUP_1, "board-1"].into());
    loop {
        board_1.set("Loop".into(), "First".into());
        for x in 1..=10 {
            counter_1.inc(1);
            counter_2.inc(10);
            counter_3.inc(100);
            pulse_1.add(x as f32);
            thread::sleep(Duration::from_secs(1));
        }
        board_1.set("Loop".into(), "Second".into());
        let pulse_2 = PulseFrameTracer::new([PACKAGE_1, DASHBOARD_2, GROUP_1, "pulse-2"].into());
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

use anyhow::Error;
use rillrate::*;
use std::thread;
use std::time::Duration;

const PACKAGE_1: &str = "package-1";
const DASHBOARD_1: &str = "dashboard-1";
const DASHBOARD_2: &str = "dashboard-2";
const DASHBOARD_I: &str = "issues";

const GROUP_1: &str = "group-1";
const GROUP_2: &str = "group-2";

pub fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    let _handle = rillrate::start();

    // Special tracers for checking issues:
    // 1. If `Pulse` has no data a range become intinite and UI app is stucked.
    let _pulse_empty = Pulse::new(
        [PACKAGE_1, DASHBOARD_I, GROUP_1, "pulse-empty"].into(),
        None,
    );
    let long_board = Board::new([PACKAGE_1, DASHBOARD_I, GROUP_2, "long-board"].into());
    long_board.set(
        "Very Long Long Long Long Long Long Long Key",
        "Very Long Long Long Long Long Long Long Long Long Long Value",
    );
    long_board.set(
        "Very Long Long Long Long Long Long Long Key1",
        "Very Long Long Long Long Long Long Long Long Long Long Value",
    );
    long_board.set(
        "Very Long Long Long Long Long Long Long Key2",
        "Very Long Long Long Long Long Long Long Long Long Long Value",
    );
    long_board.set(
        "Very Long Long Long Long Long Long Long Key3",
        "Very Long Long Long Long Long Long Long Long Long Long Value",
    );
    long_board.set(
        "Very-Long-Long-Long-Long-Long-Long-Long-Key3",
        "Very-Long-Long-Long-Long-Long-Long-Long-Long-Long-Long-Value",
    );
    long_board.set(
        "Very::Long::Long::Long::Long::Long::Long::Long::Key3",
        "Very::Long::Long::Long::Long::Long::Long::Long::Long::Long::Long::Value",
    );

    // === The main part ===
    // TODO: Improve that busy paths declarations...
    let counter_1 = Counter::new([PACKAGE_1, DASHBOARD_1, GROUP_1, "counter-1"].into(), true);
    let counter_2 = Counter::new([PACKAGE_1, DASHBOARD_1, GROUP_1, "counter-2"].into(), true);
    let counter_3 = Counter::new([PACKAGE_1, DASHBOARD_1, GROUP_1, "counter-3"].into(), true);
    let gauge_1 = Gauge::new(
        [PACKAGE_1, DASHBOARD_1, GROUP_1, "gauge-1"].into(),
        None,
        true,
    );
    let gauge_2 = Gauge::new(
        [PACKAGE_1, DASHBOARD_1, GROUP_1, "gauge-2"].into(),
        None,
        true,
    );
    let pulse_1 = Pulse::new([PACKAGE_1, DASHBOARD_2, GROUP_1, "pulse-1"].into(), None);
    let board_1 = Board::new([PACKAGE_1, DASHBOARD_2, GROUP_2, "board-1"].into());
    loop {
        board_1.set("Loop", "First");
        for x in 1..=10 {
            gauge_1.set(x as f64);
            counter_1.inc(1);
            counter_2.inc(10);
            counter_3.inc(100);
            pulse_1.add(x as f32);
            thread::sleep(Duration::from_secs(1));
        }
        board_1.set("Loop", "Second");
        let pulse_2 = Pulse::new([PACKAGE_1, DASHBOARD_2, GROUP_1, "pulse-2"].into(), None);
        for x in 1..=50 {
            gauge_2.set(x as f64);
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

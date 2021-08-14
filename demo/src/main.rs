use anyhow::Error;
use rillrate::gauge::GaugeSpec;
use rillrate::range::Range;
use rillrate::table::{Col, Row};
use rillrate::*;
use tokio::time::{sleep, Duration};

const PACKAGE_1: &str = "package-1";
const DASHBOARD_1: &str = "dashboard-1";
const DASHBOARD_2: &str = "dashboard-2";
const DASHBOARD_I: &str = "issues";

const GROUP_1: &str = "group-1";
const GROUP_2: &str = "group-2";
const GROUP_3: &str = "group-3";

const FIRST_LIMIT: usize = 10;
const SECOND_LIMIT: usize = 50;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    rillrate::install("demo")?;

    // Special tracers for checking issues:
    // 1. If `Pulse` has no data a range become intinite and UI app is stucked.
    let _pulse_empty = Pulse::new([PACKAGE_1, DASHBOARD_I, GROUP_1, "pulse-empty"], None);
    let long_board = Board::new([PACKAGE_1, DASHBOARD_I, GROUP_2, "long-board"]);
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

    let link = Link::new();
    link.sender();
    let click = Click::new(
        [PACKAGE_1, DASHBOARD_1, GROUP_1, "click-1"],
        "Click Me!",
        link.sender(),
    );
    tokio::spawn(async move {
        let mut rx = link.receiver();
        while let Some(envelope) = rx.recv().await {
            if let Some(action) = envelope.activity.to_action() {
                log::warn!("ACTION: {:?}", action);
                click.clicked();
            }
        }
    });

    let link = Link::new();
    link.sender();
    let switch = Switch::new(
        [PACKAGE_1, DASHBOARD_1, GROUP_1, "switch-1"],
        "Switch Me!",
        link.sender(),
    );
    tokio::spawn(async move {
        let mut rx = link.receiver();
        while let Some(envelope) = rx.recv().await {
            if let Some(action) = envelope.activity.to_action() {
                log::warn!("ACTION: {:?}", action);
                switch.turn(action.turn_on);
            }
        }
    });

    let link = Link::new();
    link.sender();
    let slider = Slider::new(
        [PACKAGE_1, DASHBOARD_1, GROUP_1, "slider-1"],
        "Slide Me!",
        100.0,
        5_000.0,
        100.0,
        link.sender(),
    );
    tokio::spawn(async move {
        let mut rx = link.receiver();
        while let Some(envelope) = rx.recv().await {
            if let Some(action) = envelope.activity.to_action() {
                log::warn!("ACTION: {:?}", action);
                slider.set(action.new_value);
            }
        }
    });

    let link = Link::new();
    link.sender();
    let selector = Selector::new(
        [PACKAGE_1, DASHBOARD_1, GROUP_1, "selector-1"],
        "Select Me!",
        vec!["One".into(), "Two".into(), "Three".into()],
        link.sender(),
    );
    tokio::spawn(async move {
        let mut rx = link.receiver();
        while let Some(envelope) = rx.recv().await {
            if let Some(action) = envelope.activity.to_action() {
                log::warn!("ACTION: {:?}", action);
                selector.select(action.new_selected);
            }
        }
    });

    // === The main part ===
    // TODO: Improve that busy paths declarations...
    let counter_1 = Counter::new([PACKAGE_1, DASHBOARD_1, GROUP_1, "counter-1"], true);
    let counter_2 = Counter::new([PACKAGE_1, DASHBOARD_1, GROUP_1, "counter-2"], true);
    let counter_3 = Counter::new([PACKAGE_1, DASHBOARD_1, GROUP_1, "counter-3"], true);
    let gauge_1_spec = GaugeSpec {
        pull_ms: None,
        range: Range::new(0.0, FIRST_LIMIT as f64),
    };
    let gauge_1 = Gauge::new(
        [PACKAGE_1, DASHBOARD_1, GROUP_1, "gauge-1"],
        Some(gauge_1_spec),
    );
    let gauge_2_spec = GaugeSpec {
        pull_ms: None,
        range: Range::new(0.0, SECOND_LIMIT as f64),
    };
    let gauge_2 = Gauge::new(
        [PACKAGE_1, DASHBOARD_1, GROUP_1, "gauge-2"],
        Some(gauge_2_spec),
    );
    let pulse_1 = Pulse::new([PACKAGE_1, DASHBOARD_2, GROUP_1, "pulse-1"], None);
    let board_1 = Board::new([PACKAGE_1, DASHBOARD_2, GROUP_2, "board-1"]);
    let histogram_1 = Histogram::new(
        [PACKAGE_1, DASHBOARD_2, GROUP_2, "histogram-1"],
        vec![10.0, 20.0, 100.0, 500.0],
    );
    histogram_1.add(120.0);
    histogram_1.add(11.0);

    // TABLE
    let my_table = Table::new(
        [PACKAGE_1, DASHBOARD_2, GROUP_3, "table-1"],
        vec![(Col(0), "Thread"), (Col(1), "State")],
    );
    for i in 1..=5 {
        let tbl = my_table.clone();
        let tname = format!("task-{}", i);
        tbl.add_row(Row(i));
        tbl.set_cell(Row(i), Col(0), &tname);
        tokio::spawn(async move {
            loop {
                tbl.set_cell(Row(i), Col(1), "wait 1");
                sleep(Duration::from_millis(100 * i)).await;
                tbl.set_cell(Row(i), Col(1), "wait 2");
                sleep(Duration::from_millis(500)).await;
            }
        });
    }

    loop {
        board_1.set("Loop", "First");
        for x in 1..=FIRST_LIMIT {
            gauge_1.set(x as f64);
            counter_1.inc(1);
            counter_2.inc(10);
            counter_3.inc(100);
            histogram_1.add(12.0);
            pulse_1.add(x as f64);
            sleep(Duration::from_secs(1)).await;
        }
        board_1.set("Loop", "Second");
        let pulse_2 = Pulse::new([PACKAGE_1, DASHBOARD_2, GROUP_1, "pulse-2"], None);
        for x in 1..=SECOND_LIMIT {
            gauge_2.set(x as f64);
            counter_1.inc(1);
            counter_2.inc(10);
            counter_3.inc(100);
            histogram_1.add(84.0);
            pulse_1.add(x as f64);
            pulse_2.add(x as f64);
            sleep(Duration::from_millis(500 - x as u64 * 10)).await;
        }
        sleep(Duration::from_secs(1)).await;
    }
    //rillrate::uninstall()?;
}

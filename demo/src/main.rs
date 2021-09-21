use anyhow::Error;
use rillrate::basis::Layout;
use rillrate::prime::table::{Col, Row};
use rillrate::prime::*;
use tokio::time::{sleep, Duration};

const FIRST_LIMIT: usize = 10;
const SECOND_LIMIT: usize = 50;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    rillrate::install("demo")?;

    let mut layout = Layout::new("Main Layout");
    layout.add_item((0, 0), (10, 10), "app.dashboard-1.pulses.pulse-1");
    layout.register();

    // Special tracers for checking issues:
    // 1. If `Pulse` has no data a range become intinite and UI app is stucked.
    let _pulse_empty = Pulse::new(
        "app.issues.all.pulse-empty",
        Default::default(),
        PulseOpts::default(),
    );
    let long_board = Board::new(
        "app.issues.all.long-board",
        Default::default(),
        BoardOpts::default(),
    );
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

    let alert = Alert::new("app.dashboard-1.hidden.alert", AlertOpts::default());

    let click = Click::new(
        "app.dashboard-1.controls-2.click-1",
        ClickOpts::default().label("Click Me!"),
    );
    let this = click.clone();
    let alert_cloned = alert.clone();
    click.sync_callback(move |envelope| {
        if let Some(action) = envelope.action {
            log::warn!("ACTION: {:?}", action);
            this.apply();
            // TODO: Rename to `feed`
            alert_cloned.notify("Instant alert!");
        }
        Ok(())
    });

    let switch = Switch::new(
        "app.dashboard-1.controls-2.switch-1",
        SwitchOpts::default().label("Switch Me!"),
    );
    let this = switch.clone();
    switch.sync_callback(move |envelope| {
        if let Some(action) = envelope.action {
            log::warn!("ACTION: {:?}", action);
            this.apply(action);
        }
        Ok(())
    });

    let slider = Slider::new(
        "app.dashboard-1.controls.slider-1",
        SliderOpts::default()
            .label("Slide Me!")
            .min(100)
            .max(5_000)
            .step(100),
    );
    let this = slider.clone();
    slider.sync_callback(move |envelope| {
        if let Some(action) = envelope.action {
            log::warn!("ACTION: {:?}", action);
            this.apply(action);
        }
        Ok(())
    });

    let input = Input::new(
        "app.dashboard-1.controls.input-1",
        InputOpts::default().label("Input value"),
    );
    //let this = input.clone();
    input.sync_callback(move |envelope| {
        if let Some(action) = envelope.action {
            log::warn!("ACTION: {:?}", action);
        }
        Ok(())
    });

    let selector = Selector::new(
        "app.dashboard-1.controls.selector-1",
        SelectorOpts::default()
            .label("Select Me!")
            .options(["One", "Two", "Three"]),
    );
    let this = selector.clone();
    selector.sync_callback(move |envelope| {
        if let Some(action) = envelope.action {
            log::warn!("ACTION: {:?}", action);
            this.apply(action);
        }
        Ok(())
    });

    // === The main part ===
    // TODO: Improve that busy paths declarations...
    let counter_1 = Counter::new(
        "app.dashboard-1.counters.counter-1",
        Default::default(),
        CounterOpts::default(),
    );
    let counter_2 = Counter::new(
        "app.dashboard-1.counters.counter-2",
        Default::default(),
        CounterOpts::default(),
    );
    let counter_3 = Counter::new(
        "app.dashboard-1.counters.counter-3",
        Default::default(),
        CounterOpts::default(),
    );

    let gauge_1 = Gauge::new(
        "app.dashboard-1.gauges.gauge-1",
        Default::default(),
        GaugeOpts::default().min(0.0).max(FIRST_LIMIT as f64),
    );

    let gauge_2 = Gauge::new(
        "app.dashboard-1.gauges.gauge-2",
        Default::default(),
        GaugeOpts::default().min(0.0).max(SECOND_LIMIT as f64),
    );

    let pulse_1 = Pulse::new(
        "app.dashboard-1.pulses.pulse-1",
        Default::default(),
        PulseOpts::default(),
    );
    let board_1 = Board::new(
        "app.dashboard-1.others.board-1",
        Default::default(),
        BoardOpts::default(),
    );
    let histogram_1 = Histogram::new(
        "app.dashboard-1.others.histogram-1",
        Default::default(),
        HistogramOpts::default().levels([10, 20, 100, 500]),
    );
    histogram_1.add(120.0);
    histogram_1.add(11.0);

    // TABLE
    let my_table = Table::new(
        "app.dashboard-1.z-last.table-1",
        Default::default(),
        TableOpts::default().columns([(0, "Thread".into()), (1, "State".into())]),
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

    tokio::spawn(subcrate::random_pulse());

    let live_text = LiveText::new(
        "app.dashboard-1.a-first.live-text",
        Default::default(),
        LiveTextOpts::default(),
    );

    let live_tail = LiveTail::new(
        "app.dashboard-1.a-first.live-tail",
        Default::default(),
        LiveTailOpts::default(),
    );

    let mut inner_counter = 0;
    loop {
        inner_counter += 1;
        live_text.set(format!(
            "This is a **markdown** text. Iteration: {}.",
            inner_counter
        ));
        board_1.set("Loop", "First");
        live_tail.log_now(module_path!(), "INFO", "Loop 1");
        for x in 1..=FIRST_LIMIT {
            live_tail.log_now(module_path!(), "DEBUG", format!("Line {}", x));
            gauge_1.set(x as f64);
            counter_1.inc(1);
            counter_2.inc(10);
            counter_3.inc(100);
            histogram_1.add(12.0);
            pulse_1.push(x as f64);
            sleep(Duration::from_secs(1)).await;
        }
        board_1.set("Loop", "Second");
        let pulse_2 = Pulse::new(
            "app.dashboard-1.pulses.pulse-2",
            Default::default(),
            PulseOpts::default().min(0.0).max(20.0).higher(true),
        );
        live_tail.log_now(module_path!(), "INFO", "Loop 2");
        for x in 1..=SECOND_LIMIT {
            live_tail.log_now(module_path!(), "DEBUG", format!("Line {}", x));
            gauge_2.set(x as f64);
            counter_1.inc(1);
            counter_2.inc(10);
            counter_3.inc(100);
            histogram_1.add(84.0);
            pulse_1.push(x as f64);
            pulse_2.push(x as f64);
            sleep(Duration::from_millis(500 - x as u64 * 10)).await;
        }
        alert.notify("Both loops ended!");
        sleep(Duration::from_secs(1)).await;
    }
    //rillrate::uninstall()?;
}

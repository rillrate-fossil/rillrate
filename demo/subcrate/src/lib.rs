use rand::Rng;
use rillrate::*;
use tokio::time::{sleep, Duration};

pub async fn random_pulse() {
    let pulse = Pulse::new(
        "subcrate.dashboard.all.pulse",
        Default::default(),
        PulseOpts::default().min(0).max(100),
    );
    loop {
        let value = rand::thread_rng().gen_range(0..100);
        pulse.push(value as f64);
        sleep(Duration::from_millis(200 + value)).await;
    }
}

use anyhow::Error;
use rillrate::{Click, Counter, RillRate};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    let _rillrate = RillRate::from_env("watch-click-example")?;
    tokio::spawn(counter());
    let mut shutdown = Click::create("shutdown", "Shutdown".into())?;
    shutdown.watch_click().await?;
    Ok(())
}

async fn counter() -> Result<(), Error> {
    let counter = Counter::create("counter")?;
    loop {
        counter.inc(1.0);
        sleep(Duration::from_millis(500)).await;
    }
}

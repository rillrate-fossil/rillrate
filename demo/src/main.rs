use anyhow::Error;
use std::thread;
use std::time::Duration;

pub fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    let handle = rillrate::start();
    thread::sleep(Duration::from_secs(20));
    Ok(())
}

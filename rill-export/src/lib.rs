mod actors;
mod exporters;

use anyhow::Error;
use std::thread;

pub fn start_node() -> Result<(), Error> {
    thread::Builder::new()
        .name("rill-export".into())
        .spawn(|| actors::runtime::entrypoint())?;
    Ok(())
}

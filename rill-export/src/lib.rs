mod actors;
mod exporters;

use anyhow::Error;
use meio::linkage::bridge;
use std::thread;

pub fn start_node() -> Result<(), Error> {
    let (left, right) = bridge::channel();
    thread::Builder::new()
        .name("rill-export".into())
        .spawn(move || actors::runtime::entrypoint(left))?;
    Ok(())
}

mod actors;
mod exporters;

use anyhow::Error;
use std::thread;

pub struct RillExport {}

impl RillExport {
    pub fn start() -> Result<Self, Error> {
        thread::Builder::new()
            .name("rill-export".into())
            .spawn(move || actors::runtime::entrypoint())?;
        Ok(Self {})
    }
}

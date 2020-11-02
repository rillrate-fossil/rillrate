use anyhow::Error;
use rill::provider::ProviderCell;
use std::thread;
use std::time::Duration;

static RILL: ProviderCell = ProviderCell::new(std::module_path!());
static ALT: ProviderCell = ProviderCell::new("alternative::module");

fn main() -> Result<(), Error> {
    rill::install()?;
    rill::bind_all(&[&RILL, &ALT]);
    loop {
        RILL.log("Rill Data".into());
        ALT.log("Alt Data".into());
        thread::sleep(Duration::from_millis(10));
    }
}

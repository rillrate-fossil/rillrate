use anyhow::Error;
use rill::provider::ProviderCell;
use std::thread;
use std::time::Duration;

static RILL: ProviderCell = ProviderCell::new(std::module_path!());

fn main() -> Result<(), Error> {
    rill::install()?;
    rill::bind_all(&[&RILL]);
    loop {
        RILL.log("Data".into());
        thread::sleep(Duration::from_millis(500));
    }
}

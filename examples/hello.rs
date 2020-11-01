use anyhow::Error;
use rill::provider::ProviderCell;

static RILL: ProviderCell = ProviderCell::new(std::module_path!());

fn main() -> Result<(), Error> {
    rill::install()?;
    rill::bind_all(&[&RILL]);
    RILL.log("Data".into());
    Ok(())
}

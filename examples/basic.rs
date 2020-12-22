use anyhow::Error;
use rill::Rill;

fn main() -> Result<(), Error> {
    env_logger::try_init()?;
    let rill = Rill::install("basic-example")?;
    /* TODO:
    rill.add_exporter(PrometheusExporter::new());
    */
    Ok(())
}

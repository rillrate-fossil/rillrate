use anyhow::Error;
use std::env;

pub fn embed_config() -> Result<(), Error> {
    // TODO: Move to the separate create (rate-config)
    let root = env::var("CARGO_MANIFEST_DIR")?;
    let out = env::var("OUT_DIR")?;
    let source = format!("{}/config", root);
    let dest = format!("{}/config.tar.gz", out);
    rate_core::assets::build::pack(&source, &dest)?;
    println!("cargo:rustc-env=RR_CONFIG={}", dest);
    Ok(())
}

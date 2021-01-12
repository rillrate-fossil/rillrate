mod env;

pub use rill;
pub use rill::prelude::*;
pub use rill_export;
pub use rill_protocol as protocol;

use anyhow::Error;
use rill::Rill;
use rill_export::RillExport;

/// `RillRate` tracer.
#[derive(Debug)]
pub struct RillRate {
    _rill_export: Option<RillExport>,
    _rill: Rill,
}

impl RillRate {
    /// Create a new instance configured by env vars.
    pub fn from_env(default_name: &str) -> Result<Self, Error> {
        let config_path = Some(env::config());
        let mut _rill_export = None;
        if env::standalone() {
            _rill_export = Some(RillExport::start(config_path)?);
        }
        let name = env::name().unwrap_or_else(|| default_name.into());
        let _rill = Rill::install(name)?;
        Ok(Self {
            _rill_export,
            _rill,
        })
    }

    // TODO: Add `from_config` method
}

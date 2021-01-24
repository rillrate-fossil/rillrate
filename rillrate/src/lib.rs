mod env;

pub use rill;
pub use rill::prelude::*;
pub use rill_export;
pub use rill_protocol as protocol;

use anyhow::Error;
use rill::Rill;
use rill_export::RillExport;

/// `RillRate` provider.
#[derive(Debug)]
pub struct RillRate {
    _rill_export: Option<RillExport>,
    _rill: Rill,
}

impl RillRate {
    /// Create a new instance of `RillRate` provider that configured by env vars.
    pub fn from_env(default_name: &str) -> Result<Self, Error> {
        let config_path = Some(env::config());
        let mut _rill_export = None;
        let node = {
            if let Some(node) = env::node() {
                node
            } else {
                // TODO: When started it gives an address where it has binded to
                _rill_export = Some(RillExport::start(config_path)?);
                "127.0.0.1:1636".into()
            }
        };
        let name = env::name().unwrap_or_else(|| default_name.into());
        let _rill = Rill::install(node, name)?;
        Ok(Self {
            _rill_export,
            _rill,
        })
    }

    // TODO: Add `from_config` method
}

mod env;
mod supervisor;
mod tracers;

pub use rill;
pub use rill_export;
pub use rill_protocol as protocol;
pub use tracers::*;

use anyhow::Error;
use meio::thread::ScopedRuntime;

pub struct RillRate {
    _scoped: ScopedRuntime,
}

impl RillRate {
    pub fn from_env(app_name: impl ToString) -> Result<Self, Error> {
        use supervisor::RillRate;
        let actor = RillRate::new(app_name.to_string());
        let _scoped = meio::thread::spawn(actor)?;
        Ok(Self { _scoped })
    }
}

// Not necessary in `rillrate`, because it parses all
// names automatically.
// pub use protocol::provider::{EntryId, Path};

/*
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
        let name = env::name(Some(default_name.into()));
        let with_meta = env::meta();
        let _rill = Rill::install(node, name, with_meta)?;
        Ok(Self {
            _rill_export,
            _rill,
        })
    }

    // TODO: Add `from_config` method
}
*/

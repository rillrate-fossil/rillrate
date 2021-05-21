mod actor;
pub use actor::RillConnector;
pub(crate) use actor::RillSender;

use crate::distributor::ParcelDistributor;
use once_cell::sync::Lazy;

/// It used by tracers to register them into the state.
pub(crate) static DISTRIBUTOR: Lazy<ParcelDistributor<RillConnector>> =
    Lazy::new(ParcelDistributor::new);

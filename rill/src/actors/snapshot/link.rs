use super::SnapshotTracker;
use derive_more::From;
use meio::prelude::Address;

#[derive(Debug, From)]
pub struct SnapshotTrackerLink {
    address: Address<SnapshotTracker>,
}

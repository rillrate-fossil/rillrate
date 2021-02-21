use super::RillWorker;
use derive_more::From;
use meio::prelude::Address;

#[derive(Debug, From)]
pub struct RillWorkerLink {
    address: Address<RillWorker>,
}

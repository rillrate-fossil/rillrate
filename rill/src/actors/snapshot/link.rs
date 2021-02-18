use super::SnapshotWorker;
use crate::tracers::tracer::FlowReceiver;
use anyhow::Error;
use derive_more::From;
use meio::prelude::{Action, Address};
use rill_protocol::provider::Description;
use std::sync::Arc;

#[derive(Debug, From)]
pub struct SnapshotLink {
    address: Address<SnapshotWorker>,
}

pub(super) struct AttachTracer {
    pub description: Arc<Description>,
    pub receiver: FlowReceiver,
}

impl Action for AttachTracer {}

impl SnapshotLink {
    pub async fn attach_tracer(
        &mut self,
        description: Arc<Description>,
        receiver: FlowReceiver,
    ) -> Result<(), Error> {
        let msg = AttachTracer {
            description,
            receiver,
        };
        self.address.act(msg).await
    }
}

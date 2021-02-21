use super::CounterRecorder;
use anyhow::Error;
use derive_more::From;
use meio::prelude::{Action, Address};
use rill_protocol::provider::ProviderReqId;

#[derive(Debug, From)]
pub struct CounterLink {
    address: Address<CounterRecorder>,
}

pub(super) struct ControlStream {
    pub direct_id: ProviderReqId,
    pub active: bool,
}

impl Action for ControlStream {}

impl CounterLink {
    pub async fn control_stream(
        &mut self,
        direct_id: ProviderReqId,
        active: bool,
    ) -> Result<(), Error> {
        let msg = ControlStream { direct_id, active };
        self.address.act(msg).await
    }
}

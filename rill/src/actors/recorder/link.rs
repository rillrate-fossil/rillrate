use super::Recorder;
use crate::tracers::tracer::TracerEvent;
use anyhow::Error;
use meio::prelude::{Action, ActionRecipient, Address};
use rill_protocol::provider::ProviderReqId;

#[derive(Debug)]
pub struct RecorderLink {
    recipient: Box<dyn ActionRecipient<ControlStream>>,
}

impl<T: TracerEvent> From<Address<Recorder<T>>> for RecorderLink {
    fn from(address: Address<Recorder<T>>) -> Self {
        Self {
            recipient: address.into(),
        }
    }
}

pub(super) struct ControlStream {
    pub direct_id: ProviderReqId,
    pub active: bool,
}

impl Action for ControlStream {}

impl RecorderLink {
    pub async fn control_stream(
        &mut self,
        direct_id: ProviderReqId,
        active: bool,
    ) -> Result<(), Error> {
        let msg = ControlStream { direct_id, active };
        self.recipient.act(msg).await
    }
}

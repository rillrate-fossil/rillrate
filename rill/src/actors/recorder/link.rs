use super::Recorder;
use crate::actors::worker::RillSender;
use crate::tracers::tracer::TracerEvent;
use anyhow::Error;
use meio::prelude::{Action, ActionRecipient, Address};
use rill_protocol::provider::ProviderReqId;

#[derive(Debug)]
pub(crate) struct RecorderLink {
    // TODO: Join them with `DoubleActionRecipient`
    control_recipient: Box<dyn ActionRecipient<ControlStream>>,
    connection_recipient: Box<dyn ActionRecipient<ConnectionChanged>>,
}

impl<T: TracerEvent> From<Address<Recorder<T>>> for RecorderLink {
    fn from(address: Address<Recorder<T>>) -> Self {
        Self {
            control_recipient: address.clone().into(),
            connection_recipient: address.into(),
        }
    }
}

pub(super) enum ConnectionChanged {
    Connected {
        sender: RillSender,
    },
    /// Used to drop all subscribers
    Disconnected,
}

impl Action for ConnectionChanged {}

impl RecorderLink {
    pub async fn connected(&mut self, sender: RillSender) -> Result<(), Error> {
        let msg = ConnectionChanged::Connected { sender };
        self.connection_recipient.act(msg).await
    }
}

impl RecorderLink {
    pub async fn disconnected(&mut self) -> Result<(), Error> {
        let msg = ConnectionChanged::Disconnected;
        self.connection_recipient.act(msg).await
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
        self.control_recipient.act(msg).await
    }
}

use super::Recorder;
use crate::actors::worker::RillSender;
use anyhow::Error;
use meio::{Action, ActionRecipient, Address};
use rill_protocol::flow::core;
use rill_protocol::io::provider::{ProviderReqId, RecorderRequest};

/// COOL SOLUTION!
trait Recipient
where
    Self: ActionRecipient<DoRecorderRequest>,
    Self: ActionRecipient<ConnectionChanged>,
{
}

impl<T> Recipient for T
where
    T: ActionRecipient<DoRecorderRequest>,
    T: ActionRecipient<ConnectionChanged>,
{
}

#[derive(Debug)]
pub(crate) struct RecorderLink {
    recipient: Box<dyn Recipient>,
}

impl<T: core::Flow> From<Address<Recorder<T>>> for RecorderLink {
    fn from(address: Address<Recorder<T>>) -> Self {
        Self {
            recipient: Box::new(address),
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
    // TODO: What is it? Remove?
    pub async fn connected(&mut self, sender: RillSender) -> Result<(), Error> {
        let msg = ConnectionChanged::Connected { sender };
        self.recipient.act(msg).await
    }
}

impl RecorderLink {
    pub async fn disconnected(&mut self) -> Result<(), Error> {
        let msg = ConnectionChanged::Disconnected;
        self.recipient.act(msg).await
    }
}

pub(super) struct DoRecorderRequest {
    pub direct_id: ProviderReqId,
    pub request: RecorderRequest,
}

impl Action for DoRecorderRequest {}

impl RecorderLink {
    pub async fn do_path_request(
        &mut self,
        direct_id: ProviderReqId,
        request: RecorderRequest,
    ) -> Result<(), Error> {
        let msg = DoRecorderRequest { direct_id, request };
        self.recipient.act(msg).await
    }
}

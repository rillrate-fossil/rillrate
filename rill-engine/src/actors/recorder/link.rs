use super::Recorder;
use crate::actors::worker::RillSender;
use anyhow::Error;
use meio::{Action, ActionRecipient, Address, Interaction, InteractionRecipient, InteractionTask};
use rill_protocol::flow::data;
use rill_protocol::io::provider::{PackedFlow, PackedState, ProviderReqId, RecorderAction};

/// COOL SOLUTION!
trait Recipient: ActionRecipient<DoRecorderAction> + ActionRecipient<ConnectionChanged> {}

impl<T> Recipient for T where
    T: ActionRecipient<DoRecorderAction> + ActionRecipient<ConnectionChanged>
{
}

#[derive(Debug)]
pub(crate) struct RecorderLink {
    recipient: Box<dyn Recipient>,
}

impl<T: data::Flow> From<Address<Recorder<T>>> for RecorderLink {
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

pub(super) struct DoRecorderAction {
    pub direct_id: ProviderReqId,
    pub action: RecorderAction,
}

impl Action for DoRecorderAction {}

impl RecorderLink {
    pub async fn do_path_action(
        &mut self,
        direct_id: ProviderReqId,
        action: RecorderAction,
    ) -> Result<(), Error> {
        let msg = DoRecorderAction { direct_id, action };
        self.recipient.act(msg).await
    }
}

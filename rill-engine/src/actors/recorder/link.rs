use super::Recorder;
use crate::actors::worker::RillSender;
use anyhow::Error;
use meio::{Action, ActionRecipient, Address, Interaction, InteractionRecipient, InteractionTask};
use rill_protocol::flow::data;
use rill_protocol::io::provider::{PackedFlow, PackedState, PathAction, ProviderReqId};

/// COOL SOLUTION!
trait Recipient:
    ActionRecipient<DoPathAction> + ActionRecipient<ConnectionChanged> + InteractionRecipient<FetchInfo>
{
}

impl<T> Recipient for T where
    T: ActionRecipient<DoPathAction>
        + ActionRecipient<ConnectionChanged>
        + InteractionRecipient<FetchInfo>
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

pub(super) struct DoPathAction {
    pub direct_id: ProviderReqId,
    pub action: PathAction,
}

impl Action for DoPathAction {}

impl RecorderLink {
    pub async fn do_path_action(
        &mut self,
        direct_id: ProviderReqId,
        action: PathAction,
    ) -> Result<(), Error> {
        let msg = DoPathAction { direct_id, action };
        self.recipient.act(msg).await
    }
}

// TODO: Delete
pub(crate) struct FetchInfo {
    pub with_state: bool,
}

impl Interaction for FetchInfo {
    type Output = (PackedFlow, Option<PackedState>);
}

impl RecorderLink {
    pub fn fetch_info(&mut self, with_state: bool) -> InteractionTask<FetchInfo> {
        let msg = FetchInfo { with_state };
        self.recipient.interact(msg)
    }
}

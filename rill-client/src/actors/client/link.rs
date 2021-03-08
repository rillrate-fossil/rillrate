use super::{RillClient, StateOrDelta};
use derive_more::From;
use futures::task::{Context, Poll};
use futures::{channel::mpsc, Stream};
use meio::{Address, InstantAction, Interaction, InteractionTask};
use rill_protocol::io::client::ClientReqId;
use rill_protocol::io::provider::Path;
use std::pin::Pin;

#[derive(Debug, From)]
pub struct ClientLink {
    address: Address<RillClient>,
}

pub struct Subscription {
    req_id: ClientReqId,
    receiver: mpsc::Receiver<StateOrDelta>,
    client: Address<RillClient>,
}

impl Stream for Subscription {
    type Item = StateOrDelta;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.receiver).poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.receiver.size_hint()
    }
}

impl Subscription {
    pub(super) fn new(
        req_id: ClientReqId,
        receiver: mpsc::Receiver<StateOrDelta>,
        client: Address<RillClient>,
    ) -> Self {
        Self {
            req_id,
            receiver,
            client,
        }
    }
}

pub(crate) struct UnsubscribeFromPath {
    pub req_id: ClientReqId,
}

impl InstantAction for UnsubscribeFromPath {}

/// Asks to close a stream when receiver was closed.
impl Drop for Subscription {
    fn drop(&mut self) {
        let msg = UnsubscribeFromPath {
            req_id: self.req_id,
        };
        if let Err(err) = self.client.instant(msg) {
            log::error!("Can't unsubscribe {:?}: {}", self.req_id, err);
        }
    }
}

pub struct SubscribeToPath {
    pub path: Path,
}

impl Interaction for SubscribeToPath {
    type Output = Subscription;
}

impl ClientLink {
    pub fn subscribe_to_path(&mut self, path: Path) -> InteractionTask<SubscribeToPath> {
        let msg = SubscribeToPath { path };
        self.address.interact(msg)
    }
}

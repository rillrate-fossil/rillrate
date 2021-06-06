use super::RillClient;
use derive_more::From;
use meio::{Address, Interaction};
use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::Path;
use std::marker::PhantomData;
use tokio::sync::mpsc;

#[derive(From)]
pub struct RillClientLink {
    address: Address<RillClient>,
}

struct ConnectTo<T: Flow> {
    path: Path,
    _flow: PhantomData<T>,
}

pub struct FlowReceiver<T: Flow> {
    state: T,
    receiver: mpsc::Receiver<T::Event>,
    sender: mpsc::Sender<T::Action>,
}

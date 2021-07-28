use crate::actors::client_session::ClientSession;
use crate::actors::supervisor::Supervisor;
use anyhow::Error;
use derive_more::From;
use meio::{Action, Address};
use rill_protocol::io::client::ClientServiceRequest;

#[derive(From)]
pub struct ClientLink<T: Supervisor> {
    address: Address<ClientSession<T>>,
}

/// To forward a service request to the client.
pub(super) struct ServiceOutgoing {
    pub request: ClientServiceRequest,
}

impl Action for ServiceOutgoing {}

impl<T: Supervisor> ClientLink<T> {
    pub async fn service_outgoing(&mut self, request: ClientServiceRequest) -> Result<(), Error> {
        let msg = ServiceOutgoing { request };
        self.address.act(msg).await
    }
}

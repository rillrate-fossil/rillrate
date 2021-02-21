use super::RillWorker;
use anyhow::Error;
use derive_more::From;
use meio::prelude::{Action, Address};
use rill_protocol::provider::{Direction, ProviderResponse, RillProtocol, RillToServer};

#[derive(Debug, From)]
pub struct RillWorkerLink {
    address: Address<RillWorker>,
}

pub(crate) struct SendResponse {
    pub direction: Direction<RillProtocol>,
    pub response: RillToServer,
}

impl Action for SendResponse {}

impl RillWorkerLink {
    pub async fn send_response(
        &mut self,
        direction: Direction<RillProtocol>,
        response: RillToServer,
    ) -> Result<(), Error> {
        let msg = SendResponse {
            direction,
            response,
        };
        self.address.act(msg).await
    }
}

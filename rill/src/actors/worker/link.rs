use super::RillWorker;
use crate::tracers::tracer::{DataReceiver, TracerEvent};
use anyhow::Error;
use derive_more::From;
use meio::prelude::{Address, InstantAction};
use rill_protocol::provider::Description;
use std::sync::Arc;

/*
#[derive(Debug, From)]
pub struct RillWorkerLink {
    address: Address<RillWorker>,
}
*/

/*
// TODO: Remove it and use a upstream link directly in recorders
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
*/

#[derive(Debug, From)]
pub struct RillLink {
    address: Address<RillWorker>,
}

pub(crate) struct RegisterTracer<T> {
    pub description: Arc<Description>,
    pub receiver: DataReceiver<T>,
}

impl<T: TracerEvent> InstantAction for RegisterTracer<T> {}

impl RillLink {
    pub fn register_tracer<T: TracerEvent>(
        &self,
        description: Arc<Description>,
        receiver: DataReceiver<T>,
    ) -> Result<(), Error> {
        let msg = RegisterTracer {
            description,
            receiver,
        };
        self.address.instant(msg)
    }
}

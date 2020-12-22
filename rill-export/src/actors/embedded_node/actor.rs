use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{Actor, Bridge, Consumer, Context, StartedBy, System};
use rill::protocol::{Envelope, ProviderResponse, RillToProvider, RillToServer, ServerRequest};

pub type EmbeddedNodeBridge = Bridge<ServerRequest, ProviderResponse>;

pub struct EmbeddedNode {
    bridge: EmbeddedNodeBridge,
}

impl Actor for EmbeddedNode {
    type GroupBy = ();
}

impl EmbeddedNode {
    pub fn new(bridge: EmbeddedNodeBridge) -> Self {
        Self { bridge }
    }
}

#[async_trait]
impl StartedBy<System> for EmbeddedNode {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.bridge.bind(ctx.address())?;
        Ok(())
    }
}

#[async_trait]
impl Consumer<ProviderResponse> for EmbeddedNode {
    async fn handle(
        &mut self,
        response: ProviderResponse,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        todo!();
    }
}

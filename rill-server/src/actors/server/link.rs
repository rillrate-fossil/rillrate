use super::RillServer;
use derive_more::From;
use meio::{Actor, Address, Context, Interaction, InteractionDone, InteractionTask};
use meio_connect::server::HttpServerLink;

#[derive(Debug, From)]
pub struct ServerLink {
    address: Address<RillServer>,
}

pub struct WaitPublicEndpoint;

impl Interaction for WaitPublicEndpoint {
    type Output = HttpServerLink;
}

impl ServerLink {
    pub fn wait_public_endpoint(&self) -> InteractionTask<WaitPublicEndpoint> {
        self.address.interact(WaitPublicEndpoint)
    }
}

pub struct WaitPrivateEndpoint;

impl Interaction for WaitPrivateEndpoint {
    type Output = HttpServerLink;
}

impl ServerLink {
    pub fn wait_private_endpoint(&self) -> InteractionTask<WaitPrivateEndpoint> {
        self.address.interact(WaitPrivateEndpoint)
    }
}

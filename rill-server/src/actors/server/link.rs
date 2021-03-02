use super::RillServer;
use derive_more::From;
use meio::{Address, Interaction, InteractionTask};
use meio_connect::server::HttpServerLink;

/// Link to a server.
#[derive(Debug, From)]
pub struct ServerLink {
    address: Address<RillServer>,
}

/// The notification when a public server binded to a port.
pub struct WaitPublicEndpoint;

impl Interaction for WaitPublicEndpoint {
    type Output = HttpServerLink;
}

impl ServerLink {
    /// Interaction to wait for the public server binded.
    pub fn wait_public_endpoint(&self) -> InteractionTask<WaitPublicEndpoint> {
        self.address.interact(WaitPublicEndpoint)
    }
}

/// The notification when a private server binded to a port.
pub struct WaitPrivateEndpoint;

impl Interaction for WaitPrivateEndpoint {
    type Output = HttpServerLink;
}

impl ServerLink {
    /// Interaction to wait for the private server binded.
    pub fn wait_private_endpoint(&self) -> InteractionTask<WaitPrivateEndpoint> {
        self.address.interact(WaitPrivateEndpoint)
    }
}

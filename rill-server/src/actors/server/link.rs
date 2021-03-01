use super::RillServer;
use derive_more::From;
use meio::{Actor, Address, Context, Interaction, InteractionDone};
use meio_connect::server::HttpServerLink;

#[derive(Debug, From)]
pub struct ServerLink {
    address: Address<RillServer>,
}

pub struct WaitPublicEndpoint {}

impl Interaction for WaitPublicEndpoint {
    type Output = HttpServerLink;
}

impl ServerLink {
    pub fn wait_public_endpoint<T: Actor>(&self, ctx: &mut Context<T>, group: T::GroupBy)
    where
        T: InteractionDone<WaitPublicEndpoint>,
    {
        let msg = WaitPublicEndpoint {};
        ctx.interact(&self.address, msg, group);
    }
}

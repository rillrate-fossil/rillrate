mod external;
mod internal;
pub mod limits;

use crate::actors::client_session::{ClientSession, SessionAcl};
use crate::actors::node::Node;
use crate::actors::provider_session::ProviderSession;
use crate::actors::supervisor::{Supervisor, SupervisorLink};
use crate::connection_limiter::ConnectionLimiter;
use crate::registry::Registry;
use anyhow::Error;
use async_trait::async_trait;
use meio::{Actor, Context, InterruptedBy, StartedBy};
use meio_connect::server::HttpServerLink;
use strum::{EnumIter, IntoEnumIterator};

pub struct Router<T: Supervisor> {
    external_server: HttpServerLink,
    external_port: u16,
    internal_server: HttpServerLink,
    registry: Registry,
    global_acl: SessionAcl,

    supervisor: SupervisorLink<T>,

    active_providers: ConnectionLimiter<ProviderSession>,
    active_clients: ConnectionLimiter<ClientSession<T>>,
}

impl<T: Supervisor> Router<T> {
    pub fn new(
        supervisor: SupervisorLink<T>,
        external_server: HttpServerLink,
        external_port: u16,
        internal_server: HttpServerLink,
        global_acl: SessionAcl,
    ) -> Self {
        Self {
            external_server,
            external_port,
            internal_server,
            registry: Registry::new(),
            global_acl,
            supervisor,
            // TODO: Add GlobalLimitController
            active_providers: ConnectionLimiter::new(),
            active_clients: ConnectionLimiter::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum Group {
    Externals,
    Internals,
    Fetchers,
}

impl<T: Supervisor> Actor for Router<T> {
    type GroupBy = Group;

    fn name(&self) -> String {
        "Router".into()
    }
}

#[async_trait]
impl<T: Supervisor> StartedBy<Node<T>> for Router<T> {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(Group::iter().collect());
        self.init_internal(ctx).await?;
        self.init_external(ctx).await?;
        Ok(())
    }
}

#[async_trait]
impl<T: Supervisor> InterruptedBy<Node<T>> for Router<T> {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

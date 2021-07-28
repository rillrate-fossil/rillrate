use crate::actors::supervisor::NodeSupervisor;
use anyhow::Error;
use async_trait::async_trait;
use meio::{ActionHandler, Actor, Context, InterruptedBy, StartedBy};
use rate_core::actors::client_session::{ClientLink, ClientSession, SessionAcl};
use rate_core::actors::supervisor::link;
use rill_protocol::io::client::{AccessLevel, ClientServiceRequest};
use strum::{EnumIter, IntoEnumIterator};

pub struct NodeClientAssistant {
    link: ClientLink<NodeSupervisor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum Group {}

impl Actor for NodeClientAssistant {
    type GroupBy = Group;
}

impl NodeClientAssistant {
    pub fn new(link: ClientLink<NodeSupervisor>, _session_acl: SessionAcl) -> Self {
        Self { link }
    }
}

#[async_trait]
impl StartedBy<ClientSession<NodeSupervisor>> for NodeClientAssistant {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(Group::iter().collect());
        let request = ClientServiceRequest::AccessLevel(AccessLevel::SessionCreated);
        self.link.service_outgoing(request).await?;
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<ClientSession<NodeSupervisor>> for NodeClientAssistant {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl ActionHandler<link::ServiceIncoming> for NodeClientAssistant {
    async fn handle(
        &mut self,
        _msg: link::ServiceIncoming,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        Ok(())
    }
}

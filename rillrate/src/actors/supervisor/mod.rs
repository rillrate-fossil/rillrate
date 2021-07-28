use crate::actors::client_assistant::NodeClientAssistant;
use anyhow::Error;
use async_trait::async_trait;
use meio::{Actor, Context, InteractionHandler, InterruptedBy, StartedBy, System};
use rate_core::actors::client_session::SessionAcl;
use rate_core::actors::node::{NodeConfig, NodeLink};
use rate_core::actors::supervisor::{link, Supervisor};
use strum::{EnumIter, IntoEnumIterator};

pub struct NodeSupervisor {
    config: NodeConfig,
    global_acl: SessionAcl,
    node: Option<NodeLink<Self>>,
}

impl NodeSupervisor {
    pub fn new(config: NodeConfig) -> Self {
        Self {
            config,
            global_acl: SessionAcl::new(),
            node: None,
        }
    }
}

impl Supervisor for NodeSupervisor {
    type ClientAssistant = NodeClientAssistant;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum Group {}

impl Actor for NodeSupervisor {
    type GroupBy = Group;
}

#[async_trait]
impl StartedBy<System> for NodeSupervisor {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(Group::iter().collect());
        self.global_acl.unlock_all().await;
        Ok(())
    }
}

#[async_trait]
impl InterruptedBy<System> for NodeSupervisor {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.shutdown();
        Ok(())
    }
}

#[async_trait]
impl InteractionHandler<link::GetClientAssistant<Self>> for NodeSupervisor {
    async fn handle(
        &mut self,
        msg: link::GetClientAssistant<Self>,
        _ctx: &mut Context<Self>,
    ) -> Result<NodeClientAssistant, Error> {
        Ok(NodeClientAssistant::new(msg.link, msg.session_acl))
    }
}

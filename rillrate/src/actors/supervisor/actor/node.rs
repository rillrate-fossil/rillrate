use super::{Group, NodeSupervisor};
use anyhow::Error;
use async_trait::async_trait;
use meio::{Context, Eliminated, IdOf};
use rate_core::actors::node::Node;

impl NodeSupervisor {
    pub fn spawn_node(&mut self, ctx: &mut Context<Self>) {
        let node = Node::new(
            self.config.clone(),
            ctx.address().clone(),
            self.global_acl.clone(),
        );
        let addr = ctx.spawn_actor(node, Group::Node);
        self.node = Some(addr.link());
    }
}

#[async_trait]
impl Eliminated<Node<Self>> for NodeSupervisor {
    async fn handle(
        &mut self,
        _id: IdOf<Node<Self>>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        Ok(())
    }
}

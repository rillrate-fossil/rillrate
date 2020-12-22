use anyhow::Error;
use async_trait::async_trait;
use meio::prelude::{Actor, Context, StartedBy, System};

pub struct EmbeddedNode {}

impl Actor for EmbeddedNode {
    type GroupBy = ();
}

impl EmbeddedNode {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl StartedBy<System> for EmbeddedNode {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        todo!();
    }
}

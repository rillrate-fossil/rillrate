use crate::actors::embedded_node::EmbeddedNode;
use anyhow::Error;
use meio::prelude::System;

#[tokio::main]
pub(crate) async fn entrypoint() -> Result<(), Error> {
    let node = EmbeddedNode::new();
    let mut handle = System::spawn(node);
    handle.join().await;
    Ok(())
}

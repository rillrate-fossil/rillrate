use crate::actors::embedded_node::{EmbeddedNode, EmbeddedNodeBridge};
use anyhow::Error;
use meio::prelude::System;

#[tokio::main]
pub(crate) async fn entrypoint(bridge: EmbeddedNodeBridge) -> Result<(), Error> {
    let node = EmbeddedNode::new(bridge);
    let mut handle = System::spawn(node);
    handle.join().await;
    Ok(())
}

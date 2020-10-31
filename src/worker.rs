use super::{ControlEvent, ControlReceiver};
use anyhow::Error;
use async_trait::async_trait;
use meio::{ActionHandler, Actor, Context, Supervisor};

#[tokio::main]
pub(crate) async fn entrypoint(rx: ControlReceiver) {
    let mut handle = RillWorker::new().start(Supervisor::None);
    handle.attach(rx);
    handle.join().await;
}

struct RillWorker {}

impl Actor for RillWorker {}

impl RillWorker {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ActionHandler<ControlEvent> for RillWorker {
    async fn handle(&mut self, msg: ControlEvent, _ctx: &mut Context<Self>) -> Result<(), Error> {
        todo!();
    }
}

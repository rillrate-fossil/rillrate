use meio::{Actor, Supervisor};

#[tokio::main]
pub async fn entrypoint() {
    let mut handle = RillWorker::new().start(Supervisor::None);
    handle.join().await;
}

struct RillWorker {}

impl Actor for RillWorker {}

impl RillWorker {
    pub fn new() -> Self {
        Self {}
    }
}

use super::RillClient;
use super::RillClientLink;
use anyhow::Error;
use async_trait::async_trait;
use derive_more::From;
use meio::{ActionHandler, Context, Interact, Interaction, InteractionResponder, InteractionTask};

pub struct WaitReady;

impl Interaction for WaitReady {
    type Output = ();
}

impl RillClientLink {
    pub async fn wait_ready(&mut self) -> InteractionTask<WaitReady> {
        let msg = WaitReady;
        self.address.interact(msg)
    }
}

#[derive(From)]
pub(super) struct Notifier {
    responder: InteractionResponder<()>,
}

impl Notifier {
    fn notify(self) {
        let res = self.responder.send(Ok(()));
        if let Err(_) = res {
            log::error!("Can't notify a listener that the client is ready.");
        }
    }
}

#[async_trait]
impl ActionHandler<Interact<WaitReady>> for RillClient {
    async fn handle(
        &mut self,
        input: Interact<WaitReady>,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let notifier = Notifier::from(input.responder);
        if self.sender.is_some() {
            notifier.notify();
        } else {
            self.awaiting_clients.push_back(notifier);
        }
        Ok(())
    }
}

impl RillClient {
    pub(super) fn notify_awaiting_clients(&mut self) {
        for notifier in self.awaiting_clients.drain(..) {
            notifier.notify();
        }
    }
}

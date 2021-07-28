use super::Supervisor;
use crate::actors::client_session::{ClientLink, SessionAcl};
use anyhow::Error;
use derive_more::From;
use meio::{Action, Address, Interaction, InteractionTask};
use rill_protocol::io::client::ClientServiceResponse;

#[derive(From)]
pub struct SupervisorLink<T: Supervisor> {
    address: Address<T>,
}

impl<T: Supervisor> Clone for SupervisorLink<T> {
    fn clone(&self) -> Self {
        Self {
            address: self.address.clone(),
        }
    }
}

pub struct GetClientAssistant<T: Supervisor> {
    pub link: ClientLink<T>,
    pub session_acl: SessionAcl,
}

impl<T: Supervisor> Interaction for GetClientAssistant<T> {
    type Output = T::ClientAssistant;
}

impl<T: Supervisor> SupervisorLink<T> {
    pub fn get_client_assistant(
        &mut self,
        link: ClientLink<T>,
        session_acl: SessionAcl,
    ) -> InteractionTask<GetClientAssistant<T>> {
        let msg = GetClientAssistant { link, session_acl };
        self.address.interact(msg)
    }
}

#[derive(From)]
pub struct ClientAssistant<T: Supervisor> {
    address: Address<T::ClientAssistant>,
}

pub struct ServiceIncoming {
    pub response: ClientServiceResponse,
}

impl Action for ServiceIncoming {}

impl<T: Supervisor> ClientAssistant<T> {
    pub async fn service_incoming(&mut self, response: ClientServiceResponse) -> Result<(), Error> {
        let msg = ServiceIncoming { response };
        self.address.act(msg).await
    }
}

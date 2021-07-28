use super::link;
use crate::actors::client_session::ClientSession;
use meio::{ActionHandler, Actor, InteractionHandler, InterruptedBy, StartedBy};

pub trait Supervisor: Actor + InteractionHandler<link::GetClientAssistant<Self>> {
    type ClientAssistant: Actor
        + StartedBy<ClientSession<Self>>
        + InterruptedBy<ClientSession<Self>>
        + ActionHandler<link::ServiceIncoming>;
}

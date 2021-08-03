use super::{link, SessionAcl};
use crate::actors::provider_session::link as plink;
use crate::actors::router::Router;
use crate::actors::supervisor::link as supervisor_link;
use crate::actors::supervisor::{ClientAssistant, Supervisor, SupervisorLink};
use crate::registry::{Registry, ValidPath};
use anyhow::{anyhow, Error};
use async_trait::async_trait;
use derive_more::From;
use meio::{
    ActionHandler, Actor, Context, Eliminated, IdOf, InteractionDone, InterruptedBy, StartedBy,
    Tag, TaskEliminated, TaskError,
};
use meio_connect::{
    client::WsSender,
    server::{WsHandler, WsProcessor},
    TermReason, WsIncoming,
};
use rill_protocol::io::client::{
    ClientProtocol, ClientReqId, ClientRequest, ClientResponse, ClientServiceRequest,
    ClientServiceResponse,
};
use rill_protocol::io::provider::{FlowControl, Path, RecorderAction, RecorderRequest};
use rill_protocol::io::transport::{Envelope, ServiceEnvelope};
use std::collections::hash_map::{Entry, HashMap};
use strum::{EnumIter, IntoEnumIterator};

pub type ClientSender =
    WsSender<ServiceEnvelope<ClientProtocol, ClientResponse, ClientServiceRequest>>;

pub struct ClientSession<T: Supervisor> {
    handler: WsHandler<ClientProtocol>,
    registry: Registry,
    /// The value wrapped with option to take it for `match`ing.
    directions: HashMap<ClientReqId, Option<FlowState>>,
    finalization: bool,

    supervisor: SupervisorLink<T>,
    assistant: Option<ClientAssistant<T>>,

    global_acl: SessionAcl,
    session_acl: SessionAcl,
}

#[derive(Debug)]
enum FlowState {
    /// Client is connecting to a flow.
    Subscribing,
    /// Interrupting the subscibing process.
    Interrupting,
    /// The flow is active and alive.
    Active { link: plink::SubscriptionLink },
    /// The flow is terminating.
    Unsubscribing,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum Group {
    WsConnection,
    Interactions,
    Assistant,
}

#[async_trait]
impl<T: Supervisor> Actor for ClientSession<T> {
    type GroupBy = Group;

    fn name(&self) -> String {
        format!("ClientSession({})", self.handler.addr())
    }
}

impl<T: Supervisor> ClientSession<T> {
    pub fn new(
        supervisor: SupervisorLink<T>,
        handler: WsHandler<ClientProtocol>,
        registry: Registry,
        global_acl: SessionAcl,
    ) -> Self {
        Self {
            handler,
            registry,
            directions: HashMap::new(),
            finalization: false,
            supervisor,
            assistant: None,
            global_acl,
            session_acl: SessionAcl::new(),
        }
    }

    async fn start_graceful_shutdown(&mut self, ctx: &mut Context<Self>) {
        self.finalization = true;
        if !self.directions.is_empty() {
            self.unsubscribe_all(ctx).await;
        } else {
            ctx.shutdown();
        }
    }

    fn assistant(&mut self) -> Result<&mut ClientAssistant<T>, Error> {
        self.assistant
            .as_mut()
            .ok_or_else(|| Error::msg("no assistant attached"))
    }
}

#[async_trait]
impl<T: Supervisor> StartedBy<Router<T>> for ClientSession<T> {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        ctx.termination_sequence(Group::iter().collect());

        let worker = self.handler.worker(ctx.address().clone());
        ctx.spawn_task(worker, (), Group::WsConnection);

        let link = ctx.address().link();
        let acl = self.session_acl.clone();
        // TODO: Don't block here!
        let wait_assistant = self.supervisor.get_client_assistant(link, acl);
        ctx.spawn_task(wait_assistant, (), Group::Assistant);

        Ok(())
    }
}

#[async_trait]
impl<T: Supervisor> InteractionDone<supervisor_link::GetClientAssistant<T>, ()>
    for ClientSession<T>
{
    async fn handle(
        &mut self,
        _tag: (),
        assistant: T::ClientAssistant,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let addr = ctx.spawn_actor(assistant, Group::Assistant);
        self.assistant = Some(addr.link());
        Ok(())
    }
}

#[async_trait]
impl<T: Supervisor> InterruptedBy<Router<T>> for ClientSession<T> {
    async fn handle(&mut self, ctx: &mut Context<Self>) -> Result<(), Error> {
        self.start_graceful_shutdown(ctx).await;
        Ok(())
    }
}

#[async_trait]
impl<T: Supervisor> Eliminated<T::ClientAssistant> for ClientSession<T> {
    async fn handle(
        &mut self,
        _id: IdOf<T::ClientAssistant>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        // In case if the assistant terminates the session.
        self.start_graceful_shutdown(ctx).await;
        Ok(())
    }
}

#[async_trait]
impl<T: Supervisor> TaskEliminated<WsProcessor<ClientProtocol, Self>, ()> for ClientSession<T> {
    async fn handle(
        &mut self,
        _id: IdOf<WsProcessor<ClientProtocol, Self>>,
        _tag: (),
        _result: Result<TermReason, TaskError>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::warn!("WS DISCONNNECTED");
        self.start_graceful_shutdown(ctx).await;
        Ok(())
    }
}

impl<T: Supervisor> ClientSession<T> {
    fn resolve_aliases(&self, path: Path) -> Path {
        let mut entries = Vec::new();
        for entry_id in path.into_iter() {
            if entry_id.as_ref() == "@self" {
                entries.push(self.global_acl.id().clone());
                entries.push(self.session_acl.id().clone());
            } else if entry_id.as_ref() == "@server" {
                entries.push(self.global_acl.id().clone());
            } else {
                entries.push(entry_id);
            }
        }
        entries.into()
    }

    async fn subscribe_or_act(
        &mut self,
        direct_id: ClientReqId,
        path: Path,
        action: Option<RecorderAction>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        // SUBSCRIBING
        // TODO: Fix `ValidPath` shit
        // TODO: Return error for invalid paths
        // TODO: Use `AliasPath` instead (to prevent potential mistakes on refactoring)
        #[allow(clippy::collapsible_if)]
        if !self.session_acl.has_access_to(&path).await {
            if !self.global_acl.has_access_to(&path).await {
                //log::error!("No access to: {}", path);
                return Err(anyhow!("No access to: {}", path));
            }
        }
        //log::info!("Path {} allowed!", path);

        let resolved_path = self.resolve_aliases(path);
        let valid_path = ValidPath(resolved_path);
        let entry = self.directions.entry(direct_id);
        match entry {
            Entry::Vacant(entry) => {
                let provider = self.registry.find_provider(&valid_path).await;
                if let Some((mut link, remained_path)) = provider {
                    match action {
                        Some(action) => {
                            let sender = self.handler.sender();
                            link.action_on_path(remained_path, direct_id, sender, action)
                                .await?;
                        }
                        None => {
                            entry.insert(Some(FlowState::Subscribing));
                            let task =
                                link.subscribe(remained_path, direct_id, self.handler.sender());
                            let tag = FlowTag { req_id: direct_id };
                            ctx.track_interaction(task, tag, Group::Interactions);
                        }
                    }
                } else {
                    log::error!("Can't find flow with path: {}", &valid_path.0);
                    // TODO: Return error to the client
                }
                Ok(())
            }
            Entry::Occupied(_entry) => Err(anyhow!(
                "Attempt to subscribe twice using the same direct id: {:?}",
                direct_id
            )),
        }
    }

    // TODO: I don't like this method. Try to improve it: simplify and remove `FT` parameter.
    async fn unsubscribe<FT>(
        &mut self,
        direct_id: ClientReqId,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error>
    where
        Self: InteractionDone<plink::UnsubscribeFromPath, FT> + Actor<GroupBy = Group>,
        FT: Tag + From<ClientReqId>,
    {
        // UNSUBSCRIBING
        if let Some(state) = self.directions.get_mut(&direct_id) {
            // TODO: Don't remove it above and unsubscribe asynchronously
            match state.take() {
                Some(FlowState::Subscribing) => {
                    // Just set it to terminating and wait
                    // when the provider will return a link for unsubscribing.
                    *state = Some(FlowState::Interrupting);
                    Ok(())
                }
                Some(FlowState::Interrupting) => {
                    *state = Some(FlowState::Interrupting);
                    Err(anyhow!("Attempt to unsubscribe twice for: {:?}", direct_id))
                }
                Some(FlowState::Active { link }) => {
                    let task = link.unsubscribe();
                    let tag = FT::from(direct_id);
                    ctx.track_interaction(task, tag, Group::Interactions);
                    *state = Some(FlowState::Unsubscribing);
                    Ok(())
                }
                Some(FlowState::Unsubscribing) => {
                    *state = Some(FlowState::Unsubscribing);
                    Err(anyhow!(
                        "Attempt to unsubscribe for the terminating flow: {:?}",
                        direct_id
                    ))
                }
                None => Err(anyhow!("FATAL: Flow stucked in transition state")),
            }
        } else {
            Err(anyhow!("Client wan't subscribed to {:?}", direct_id))
        }
    }

    async fn unsubscribe_all(&mut self, ctx: &mut Context<Self>) {
        let ids: Vec<_> = self.directions.keys().cloned().collect();
        for req_id in ids {
            // TODO: Unsubscribe one-by-one to avoid any potential blocking
            if let Err(err) = self.unsubscribe::<FinalFlowTag>(req_id, ctx).await {
                log::error!("Unsubscribing of {:?} failed: {}", req_id, err);
            }
        }
    }
}

#[async_trait]
impl<T: Supervisor>
    ActionHandler<WsIncoming<ServiceEnvelope<ClientProtocol, ClientRequest, ClientServiceResponse>>>
    for ClientSession<T>
{
    async fn handle(
        &mut self,
        msg: WsIncoming<ServiceEnvelope<ClientProtocol, ClientRequest, ClientServiceResponse>>,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        log::trace!("Client request: {:?}", msg);
        //log::trace!("DIRECTIONS: {:?}", self.directions);
        match msg.0 {
            ServiceEnvelope::Envelope(envelope) => {
                let direct_id = envelope.direct_id;
                let path = envelope.data.path;
                match envelope.data.request {
                    RecorderRequest::ControlStream(control) => match control {
                        FlowControl::StartStream => {
                            self.subscribe_or_act(direct_id, path, None, ctx).await
                        }
                        FlowControl::StopStream => {
                            self.unsubscribe::<FlowTag>(direct_id, ctx).await
                        }
                    },
                    RecorderRequest::Action(action) => {
                        self.subscribe_or_act(direct_id, path, Some(action), ctx)
                            .await
                    }
                }
            }
            ServiceEnvelope::Service(service) => {
                self.assistant()?.service_incoming(service).await?;
                Ok(())
            }
        }
    }
}

#[derive(From)]
struct FlowTag {
    req_id: ClientReqId,
}

impl Tag for FlowTag {}

#[async_trait]
impl<T: Supervisor> InteractionDone<plink::SubscribeToPath, FlowTag> for ClientSession<T> {
    async fn handle(
        &mut self,
        tag: FlowTag,
        link: plink::SubscriptionLink,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let direct_id = tag.req_id;
        if let Some(state) = self.directions.get_mut(&direct_id) {
            match state.take() {
                Some(FlowState::Subscribing) => {
                    let new_state = FlowState::Active { link };
                    *state = Some(new_state);
                    Ok(())
                }
                Some(FlowState::Interrupting) => {
                    let task = link.unsubscribe();
                    let tag = FlowTag { req_id: direct_id };
                    ctx.track_interaction(task, tag, Group::Interactions);
                    *state = Some(FlowState::Unsubscribing);
                    Ok(())
                }
                // TODO: Check that below
                Some(active @ FlowState::Active { .. }) => {
                    *state = Some(active);
                    Err(anyhow!(
                        "Provider returned the second subscription link for {:?}",
                        direct_id
                    ))
                }
                Some(unsubcribing @ FlowState::Unsubscribing) => {
                    *state = Some(unsubcribing);
                    Err(anyhow!("Provider returned the second subscription link when unsubscribing for {:?}", direct_id))
                }
                None => Err(anyhow!("Transition state in subscribing interaction")),
            }
        } else {
            Err(anyhow!("awaiting direction lost"))
        }
    }

    async fn failed(
        &mut self,
        _tag: FlowTag,
        _reason: TaskError,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        // TODO: Return error to the client.
        Ok(())
    }
}

#[async_trait]
impl<T: Supervisor> InteractionDone<plink::UnsubscribeFromPath, FlowTag> for ClientSession<T> {
    async fn handle(
        &mut self,
        tag: FlowTag,
        _res: (),
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let direct_id = tag.req_id;
        if let Some(_state) = self.directions.remove(&direct_id) {
            let data = ClientResponse::Done;
            let envelope = Envelope { direct_id, data };
            let service_envelope = ServiceEnvelope::Envelope(envelope);
            self.handler.send(service_envelope);
            Ok(())
        } else {
            Err(anyhow!(
                "Client request id {:?} is not exists for unsubscribing task.",
                direct_id
            ))
        }
    }

    async fn failed(
        &mut self,
        tag: FlowTag,
        _reason: TaskError,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        InteractionDone::<plink::UnsubscribeFromPath, FlowTag>::handle(self, tag, (), ctx).await
    }
}

#[derive(From)]
struct FinalFlowTag {
    req_id: ClientReqId,
}

impl Tag for FinalFlowTag {}

#[async_trait]
impl<T: Supervisor> InteractionDone<plink::UnsubscribeFromPath, FinalFlowTag> for ClientSession<T> {
    async fn handle(
        &mut self,
        tag: FinalFlowTag,
        _res: (),
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let direct_id = tag.req_id;
        self.directions.remove(&direct_id);
        if self.directions.is_empty() {
            self.start_graceful_shutdown(ctx).await;
        }
        Ok(())
    }

    async fn failed(
        &mut self,
        tag: FinalFlowTag,
        _reason: TaskError,
        ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        InteractionDone::<plink::UnsubscribeFromPath, FinalFlowTag>::handle(self, tag, (), ctx)
            .await
    }
}

#[async_trait]
impl<T: Supervisor> ActionHandler<link::ServiceOutgoing> for ClientSession<T> {
    async fn handle(
        &mut self,
        msg: link::ServiceOutgoing,
        _ctx: &mut Context<Self>,
    ) -> Result<(), Error> {
        let service_envelope = ServiceEnvelope::Service(msg.request);
        self.handler.send(service_envelope);
        Ok(())
    }
}

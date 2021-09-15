use crate::agents::live::{LiveAgent, LiveRequest, LiveResponse, LiveStatus};
use rill_protocol::flow::core;
use rill_protocol::io::client::{ClientReqId, ClientRequest, ClientResponse};
use rill_protocol::io::provider::{FlowControl, Path, RecorderAction, RecorderRequest};
use std::fmt;
use yew::worker::{AgentLink, HandlerId};

#[derive(Debug)]
pub struct WireEnvelope<ID, T> {
    pub id: ID,
    pub data: T,
}

impl<ID, T> WireEnvelope<ID, T> {
    pub fn new(id: ID, data: T) -> Self {
        Self { id, data }
    }
}

// TODO: Don't require Debug
pub trait WireTask: fmt::Debug + 'static {
    fn on_action(&mut self, action: WireAction, context: WireContext<'_>);
}

pub enum WireAction {
    // It's separate `LiveStatus` type to make status cloneable
    Status(LiveStatus),
    Incoming(ClientResponse),
    Interrupted,
}

pub struct WireContext<'a> {
    pub who: HandlerId,
    pub req_id: ClientReqId,
    pub link: &'a mut AgentLink<LiveAgent>,
}

impl<'a> WireContext<'a> {
    fn send_to_server(&mut self, request: ClientRequest) {
        let input = LiveRequest::Forward(request);
        let envelope = WireEnvelope::new(self.req_id, input);
        self.link.send_input(envelope);
    }

    fn send_to_component(&mut self, response: ClientResponse) {
        let output = LiveResponse::Forwarded(response);
        let envelope = WireEnvelope::new(self.req_id, output);
        self.link.respond(self.who, envelope);
    }

    fn shutdown(&mut self) {
        let input = LiveRequest::DetachRuntime;
        let envelope = WireEnvelope::new(self.req_id, input);
        self.link.send_input(envelope);
    }
}

#[derive(Debug)]
pub struct Subscription {
    path: Path,
    sent: bool,
    interrupted: bool,
}

impl Subscription {
    pub fn new(path: Path) -> Self {
        Self {
            path,
            sent: false,
            interrupted: false,
        }
    }
}

impl WireTask for Subscription {
    fn on_action(&mut self, action: WireAction, mut ctx: WireContext<'_>) {
        match action {
            WireAction::Status(LiveStatus::Connected) => {}
            WireAction::Status(LiveStatus::AccessLevel(_)) => {
                if !self.sent {
                    self.sent = true;
                    if !self.interrupted {
                        let control = FlowControl::StartStream;
                        let request = RecorderRequest::ControlStream(control);
                        let request = ClientRequest {
                            path: self.path.clone(),
                            request,
                        };
                        ctx.send_to_server(request);
                    }
                }
            }
            WireAction::Status(LiveStatus::Disconnected) => {
                self.sent = false;
                // TODO: Send `Disconnected` to a Component
                //ctx.shutdown();
            }
            WireAction::Incoming(response) => {
                if !self.interrupted {
                    match &response {
                        ClientResponse::Done => {
                            self.interrupted = true;
                        }
                        ClientResponse::Error(err) => {
                            log::error!("Stream {} failed: {}", self.path, err);
                        }
                        _ => {}
                    }
                    // TODO: Try again if had an error: for the case if the server was restarted
                    // and not all providers sent when the client tried to reach them.
                    //
                    ctx.send_to_component(response);
                }
            }
            WireAction::Interrupted => {
                if self.sent && !self.interrupted {
                    self.interrupted = true;
                    let control = FlowControl::StopStream;
                    let request = RecorderRequest::ControlStream(control);
                    let request = ClientRequest {
                        path: self.path.clone(),
                        request,
                    };
                    ctx.send_to_server(request);
                    // Wait for the `End` marker or `Error` or `Disconnected`
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct GetDescription {
    sent: bool,
    path: Path,
}

impl GetDescription {
    pub fn new(path: Path) -> Self {
        Self { sent: false, path }
    }
}

impl WireTask for GetDescription {
    fn on_action(&mut self, action: WireAction, mut ctx: WireContext<'_>) {
        match action {
            WireAction::Status(LiveStatus::Connected) => {}
            WireAction::Status(LiveStatus::AccessLevel(_)) => {
                if !self.sent {
                    self.sent = true;
                    let action = RecorderAction::GetFlow;
                    let request = RecorderRequest::Action(action);
                    let request = ClientRequest {
                        path: self.path.clone(),
                        request,
                    };
                    ctx.send_to_server(request);
                }
            }
            WireAction::Status(LiveStatus::Disconnected) => {
                self.sent = false;
                // TODO: Send End to Component
                ctx.shutdown();
            }
            WireAction::Incoming(response) => {
                // TODO: Match `End`, `Error, `Disconected`
                ctx.send_to_component(response);
                ctx.shutdown();
            }
            WireAction::Interrupted => {
                // TODO: Set `drop_response/not_active` flag
                // Wait for the `End` marker or `Error` or `Disconnected`
                ctx.shutdown();
            }
        }
    }
}

#[derive(Debug)]
pub struct DoAction<T: core::Flow> {
    sent: bool,
    path: Path,
    action: T::Action,
}

impl<T: core::Flow> DoAction<T> {
    pub fn new(path: Path, action: T::Action) -> Self {
        Self {
            sent: false,
            path,
            action,
        }
    }
}

impl<T: core::Flow> WireTask for DoAction<T> {
    fn on_action(&mut self, action: WireAction, mut ctx: WireContext<'_>) {
        match action {
            WireAction::Status(LiveStatus::Connected) => {}
            WireAction::Status(LiveStatus::AccessLevel(_)) => {
                if !self.sent {
                    self.sent = true;
                    match T::pack_action(&self.action) {
                        Ok(packed_action) => {
                            let action = RecorderAction::DoAction(packed_action);
                            let request = RecorderRequest::Action(action);
                            let request = ClientRequest {
                                path: self.path.clone(),
                                request,
                            };
                            ctx.send_to_server(request);
                        }
                        Err(err) => {
                            log::error!("Can't pack an action: {}", err);
                        }
                    }
                    ctx.shutdown();
                }
            }
            WireAction::Status(LiveStatus::Disconnected) => {
                // TODO: Send End to Component
                ctx.shutdown();
            }
            WireAction::Incoming(_response) => {
                // TODO: Match `End`, `Error, `Disconected`
                ctx.shutdown();
            }
            WireAction::Interrupted => {
                // TODO: Set `drop_response/not_active` flag
                // Wait for the `End` marker or `Error` or `Disconnected`
                ctx.shutdown();
            }
        }
    }
}

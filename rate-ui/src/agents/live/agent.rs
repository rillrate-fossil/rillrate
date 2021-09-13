use super::registry::REGISTRY;
use super::wire::{WireAction, WireContext, WireEnvelope, WireTask};
use anyhow::Error;
use rill_protocol::io::client::{
    AccessLevel, ClientProtocol, ClientReqId, ClientRequest, ClientResponse, ClientServiceRequest,
    ClientServiceResponse,
};
use rill_protocol::io::transport::{Envelope, ServiceEnvelope};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use url::Url;
use yew::services::timeout::{TimeoutService, TimeoutTask};
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::worker::{Agent, AgentLink, Context, HandlerId};

struct WireRuntime {
    who: HandlerId,
    req_id: ClientReqId,
    task: Box<dyn WireTask>,
}

impl WireRuntime {
    fn wire_action(&mut self, action: WireAction, link: &mut AgentLink<LiveAgent>) {
        let context = WireContext {
            who: self.who,
            req_id: self.req_id,
            link,
        };
        self.task.on_action(action, context);
    }
}

pub enum Msg {
    WsIncoming(
        Result<ServiceEnvelope<ClientProtocol, ClientResponse, ClientServiceRequest>, Error>,
    ),
    WsStatus(WebSocketStatus),
    TryReconnect,
    // Don't add to many variants here
}

#[derive(Debug)]
pub enum LiveRequest {
    /// Created a new wire
    Wire(Box<dyn WireTask>),
    /// Interrupts a wire
    TerminateWire,

    Forward(ClientRequest),
    DetachRuntime,
}

#[derive(Debug, Clone)]
pub enum LiveResponse {
    Forwarded(ClientResponse),
    WireDone,
}

#[derive(Debug, Clone)]
pub enum LiveStatus {
    Disconnected,
    Connected,
    AccessLevel(AccessLevel),
}

impl LiveStatus {
    fn is_connected(&self) -> bool {
        !matches!(self, Self::Disconnected)
    }
}

pub struct LiveAgent {
    link: AgentLink<Self>,
    status: LiveStatus,
    ws: Option<WebSocketTask>,
    wires: HashMap<ClientReqId, WireRuntime>,
    reconnection_task: Option<TimeoutTask>,
}

impl Agent for LiveAgent {
    type Reach = Context<Self>;
    type Message = Msg;
    type Input = WireEnvelope<ClientReqId, LiveRequest>;
    type Output = WireEnvelope<ClientReqId, LiveResponse>;

    fn create(link: AgentLink<Self>) -> Self {
        let mut this = Self {
            link,
            status: LiveStatus::Disconnected,
            ws: None,
            wires: HashMap::new(),
            reconnection_task: None,
        };
        if let Err(err) = this.connect() {
            log::error!("Can't start conencting because of: {}", err);
        }
        this
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::WsIncoming(Ok(response)) => {
                //log::trace!("WS-RECV: {:?}", response);
                match response {
                    ServiceEnvelope::Envelope(envelope) => {
                        let direct_id = envelope.direct_id;
                        let action = WireAction::Incoming(envelope.data);
                        let runtime = self.wires.get_mut(&direct_id);
                        if let Some(runtime) = runtime {
                            runtime.wire_action(action, &mut self.link);
                        }
                    }
                    ServiceEnvelope::Service(service) => match service {
                        /*
                        ClientServiceRequest::Ping => {
                            let response = ClientServiceResponse::Pong;
                            let service_envelope = ServiceEnvelope::Service(response);
                            self.send_service_envelope(service_envelope);
                        }
                        */
                        ClientServiceRequest::AccessLevel(access_level) => {
                            log::info!("ACCESS LEVEL: {:?}", access_level);
                            self.status = LiveStatus::AccessLevel(access_level);
                            self.status_to_wires(self.status.clone());
                        }
                    },
                }
            }
            Msg::WsIncoming(Err(err)) => {
                log::error!("Invalid incoiming message: {}", err);
            }
            Msg::WsStatus(status) => {
                match status {
                    WebSocketStatus::Opened => {
                        log::info!("CONNECTED!");
                        self.status = LiveStatus::Connected;
                    }
                    WebSocketStatus::Closed | WebSocketStatus::Error => {
                        log::info!("DISCONNECTED!");
                        self.status = LiveStatus::Disconnected;
                        self.ws.take();
                        let duration = Duration::from_secs(5);
                        let callback = self.link.callback(|_| Msg::TryReconnect);
                        let task = TimeoutService::spawn(duration, callback);
                        self.reconnection_task = Some(task);
                    }
                }
                self.status_to_wires(self.status.clone());
            }
            Msg::TryReconnect => {
                self.reconnection_task.take();
                if let Err(err) = self.connect() {
                    log::error!("Can't reconnect because of: {}", err);
                }
            }
        }
    }

    fn handle_input(&mut self, request: Self::Input, who: HandlerId) {
        //log::trace!("LiveAgent request {:?} from {:?}", request, who);
        let id = request.id;
        match request.data {
            LiveRequest::Wire(task) => {
                let mut runtime = WireRuntime {
                    who,
                    req_id: id,
                    task,
                };
                // TODO: Do I have to send desconnected status in any case?
                if self.status.is_connected() {
                    /*
                    let action = WireAction::Status(LiveStatus::Connected);
                    runtime.wire_action(action, &mut self.link);
                    */
                    let action = WireAction::Status(self.status.clone());
                    runtime.wire_action(action, &mut self.link);
                }
                self.wires.insert(id, runtime);
            }
            LiveRequest::TerminateWire => {
                if let Some(runtime) = self.wires.get_mut(&id) {
                    let action = WireAction::Interrupted;
                    runtime.wire_action(action, &mut self.link);
                }
            }
            LiveRequest::Forward(request) => {
                let envelope = Envelope {
                    direct_id: id,
                    data: request,
                };
                let service_envelope = ServiceEnvelope::Envelope(envelope);
                self.send_service_envelope(service_envelope);
            }
            LiveRequest::DetachRuntime => {
                if let Some(runtime) = self.wires.remove(&id) {
                    let output = LiveResponse::WireDone;
                    let envelope = WireEnvelope::new(runtime.req_id, output);
                    self.link.respond(runtime.who, envelope);
                    // Nothing will be forwarded since the runtime has removed
                    REGISTRY.release(id);
                } else {
                    log::error!("Can't detach a runtime with id {:?}", id);
                }
            }
        }
    }

    fn connected(&mut self, _id: HandlerId) {
        //log::trace!("Connected to Live: {:?}", id);
    }

    fn disconnected(&mut self, _id: HandlerId) {
        //log::trace!("Disconnected from Live: {:?}", id);
    }
}

#[derive(Error, Debug)]
enum ConnectorError {
    #[error("can't get window object")]
    NoWindow,
    #[error("can't convert location to a string")]
    NoString,
}

impl LiveAgent {
    fn status_to_wires(&mut self, live_status: LiveStatus) {
        for (_req_id, runtime) in &mut self.wires {
            let action = WireAction::Status(live_status.clone());
            runtime.wire_action(action, &mut self.link);
        }
    }

    fn send_service_envelope(
        &mut self,
        service_envelope: ServiceEnvelope<ClientProtocol, ClientRequest, ClientServiceResponse>,
    ) {
        if let Some(ws) = self.ws.as_mut() {
            if self.status.is_connected() {
                //log::trace!("WS-SEND: {:?}", service_envelope);
                let data = rill_protocol::encoding::to_vec(&service_envelope);
                ws.send_binary(data);
            } else {
                log::error!(
                    "Connection not established yet for sending: {:?}",
                    service_envelope
                );
            }
        } else {
            // TODO: Add and use tasks queue
            log::error!("No connection to send: {:?}", service_envelope);
        }
    }

    fn connect(&mut self) -> Result<(), Error> {
        let mut url: Url = web_sys::window()
            .ok_or(ConnectorError::NoWindow)?
            .location()
            .to_string()
            .as_string()
            .ok_or(ConnectorError::NoString)?
            .parse()?;
        let scheme = if url.scheme().ends_with("s") {
            "wss"
        } else {
            "ws"
        };
        url.set_scheme(scheme)
            .map_err(|_| Error::msg("Can't set scheme"))?;
        url.set_path("/live/client");
        let url = url.to_string();
        log::info!("Location: {}", url);
        let callback = self.link.callback(|data: Result<Vec<u8>, Error>| {
            let res = data.and_then(|data| rill_protocol::encoding::from_slice(&data));
            Msg::WsIncoming(res)
        });
        let notification = self.link.callback(Msg::WsStatus);
        let ws = WebSocketService::connect_binary(&url, callback, notification)
            .map_err(|reason| Error::msg(reason.to_string()))?;
        self.ws = Some(ws);
        Ok(())
    }
}

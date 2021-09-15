use crate::io::codec::BinaryCodec;
use crate::io::provider::{Description, EntryId, PackedEvent, PackedState, Path, RecorderRequest};
use crate::io::transport::{DirectId, Origin, ServiceEnvelope};
use meio_protocol::Protocol;
use serde::{Deserialize, Serialize};

pub type ClientReqId = DirectId<ClientProtocol>;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ClientProtocol;

impl Protocol for ClientProtocol {
    type ToServer = ServiceEnvelope<Self, ClientRequest, ClientServiceResponse>;
    type ToClient = ServiceEnvelope<Self, ClientResponse, ClientServiceRequest>;
    type Codec = BinaryCodec;
}

impl Origin for ClientProtocol {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRequest {
    pub path: Path,
    pub request: RecorderRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientResponse {
    Declare(EntryId),
    Flow(Description),
    State(PackedState),
    Delta(PackedEvent),
    /// Stream closed/finished.
    Done,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientServiceRequest {
    //Ping,
    AccessLevel(AccessLevel),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientServiceResponse {
    //Pong,
}

/// `AccessLevel` notifies about specific stages of a session:
/// - session created (ready for pings)
/// - client can sign in or sign up
/// - client can work will all accessible flows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessLevel {
    SessionCreated,
    ReadyToAuth,
    ReadyToWork,
}

use crate::io::codec::RRCodec;
use crate::io::provider::{Description, EntryId, PackedDelta, PackedState, Path, RecorderRequest};
use crate::io::transport::{DirectId, Origin, ServiceEnvelope};
use meio_protocol::Protocol;
use serde::{Deserialize, Serialize};

pub type ClientReqId = DirectId<ClientProtocol>;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ClientProtocol;

impl Protocol for ClientProtocol {
    type ToServer = ServiceEnvelope<Self, ClientRequest, ()>;
    type ToClient = ServiceEnvelope<Self, ClientResponse, ()>;
    type Codec = RRCodec;
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
    Delta(PackedDelta),
    /// Stream closed/finished.
    Done,
    Error(String),
}

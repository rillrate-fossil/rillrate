use crate::io::codec::RRCodec;
use crate::io::provider::{EntryId, PackedDelta, PackedState, Path};
use crate::io::transport::{DirectId, Envelope, Origin};
use meio_protocol::Protocol;
use serde::{Deserialize, Serialize};

pub type ClientReqId = DirectId<ClientProtocol>;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ClientProtocol;

impl Protocol for ClientProtocol {
    type ToServer = Envelope<Self, ClientRequest>;
    // TODO: Consider to disallow broadcasts and change to ordinary `Envelope`
    type ToClient = Envelope<Self, ClientResponse>;
    type Codec = RRCodec;
}

impl Origin for ClientProtocol {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientRequest {
    ControlStream { path: Path, active: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientResponse {
    Declare(EntryId),
    State(PackedState),
    Delta(PackedDelta),
    /// Stream closed/finished.
    Done,
    Error(String),
}

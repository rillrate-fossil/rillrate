use crate::io::codec::RRCodec;
use crate::io::provider::{Description, EntryId, PackedDelta, PackedState, Path};
use crate::io::transport::{DirectId, Envelope, Origin, WideEnvelope};
use meio_protocol::Protocol;
use serde::{Deserialize, Serialize};

pub type ClientReqId = DirectId<ClientProtocol>;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ClientProtocol;

impl Protocol for ClientProtocol {
    type ToServer = Envelope<Self, ClientRequest>;
    // TODO: Consider to disallow broadcasts and change to ordinary `Envelope`
    type ToClient = WideEnvelope<Self, ClientResponse>;
    type Codec = RRCodec;
}

impl Origin for ClientProtocol {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientRequest {
    //SubscribeProviders { active: bool },
    //SubscribePaths { provider: EntryId, active: bool },
    ControlStream { path: Path, active: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientResponse {
    Declare(EntryId),
    // TODO: Replace to `State/Delta` meta stream
    Paths(Vec<Description>),
    State(PackedState),
    Delta(PackedDelta),
    /// Stream closed/finished.
    Done,
    Error(String),
}

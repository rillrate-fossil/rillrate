use crate::codec::JsonCodec;
use crate::provider::{Description, EntryId, Origin, Path, RillEvent};
use meio_protocol::Protocol;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ClientProtocol;

impl Protocol for ClientProtocol {
    type ToServer = ClientRequest;
    type ToClient = ClientResponse;
    type Codec = JsonCodec;
}

impl Origin for ClientProtocol {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientRequest {
    ControlStream { path: Path, active: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientResponse {
    Declare(EntryId),
    Paths(Vec<Description>),
    Data(Path, RillEvent),
}

use crate::codec::JsonCodec;
use crate::provider::{DirectId, Envelope, Origin, RillToProvider, RillToServer};
use meio_protocol::Protocol;
use serde::{Deserialize, Serialize};

pub type ViewReqId = DirectId<ViewProtocol>;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ViewProtocol;

impl Protocol for ViewProtocol {
    type ToServer = Envelope<Self, ViewRequest>;
    type ToClient = Envelope<Self, ViewResponse>;
    type Codec = JsonCodec;
}

impl Origin for ViewProtocol {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewRequest {
    Forward(RillToProvider),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewResponse {
    Forward(RillToServer),
}

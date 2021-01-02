use crate::codec::JsonCodec;
use crate::provider::{Envelope, Origin};
use meio_protocol::Protocol;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ViewProtocol;

impl Protocol for ViewProtocol {
    type ToServer = Envelope<Self, ViewRequest>;
    type ToClient = Envelope<Self, ViewResponse>;
    type Codec = JsonCodec;
}

impl Origin for ViewProtocol {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewResponse {}

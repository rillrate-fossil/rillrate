use crate::codec::JsonCodec;
use crate::provider::{Description, Origin, Path, RillData};
use meio_protocol::Protocol;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ViewProtocol;

impl Protocol for ViewProtocol {
    type ToServer = ViewRequest;
    type ToClient = ViewResponse;
    type Codec = JsonCodec;
}

impl Origin for ViewProtocol {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewRequest {
    ControlStream { path: Path, active: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewResponse {
    Paths(Vec<Description>),
    Data(Path, Duration, RillData),
}

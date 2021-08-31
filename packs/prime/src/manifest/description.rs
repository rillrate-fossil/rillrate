use rill_protocol::flow::core::Flow;
use rill_protocol::io::provider::{Path, StreamType};
use serde::{Deserialize, Serialize};

pub trait PackFlow: Flow {
    fn layer() -> Layer;
}

/*
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Weight {
    pub group: u16,
    pub item: u16,
}

impl From<u16> for Weight {
    fn from(value: u16) -> Self {
        let group = value / 100;
        let item = value % 100;
        Self { group, item }
    }
}
*/

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Layer {
    Visual,
    Control,
    Transparent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackFlowDescription {
    pub path: Path,
    pub layer: Layer,
    pub stream_type: StreamType,
}

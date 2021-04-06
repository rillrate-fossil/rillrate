use super::Inflow;
use crate::io::provider::StreamType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClickInflow {}

impl Inflow for ClickInflow {
    type Event = ClickEvent;

    fn stream_type() -> StreamType {
        StreamType::from("rillrate.inflow.action.click.v0")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickEvent;

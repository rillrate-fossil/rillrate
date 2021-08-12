use crate::base_control::emit_control::{EmitControlSpec, EmitControlState};
use rill_protocol::flow::core::TimedEvent;
use rill_protocol::io::provider::StreamType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickSpec;

impl EmitControlSpec for ClickSpec {
    type State = ();
    type Action = ();

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(_state: &mut Self::State, _action: TimedEvent<Self::Action>) {
        //
    }
}

pub type ClickState = EmitControlState<ClickSpec>;

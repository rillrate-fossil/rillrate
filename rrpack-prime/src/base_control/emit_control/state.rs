use rill_protocol::flow::core::{DataFraction, Flow, TimedEvent};
use rill_protocol::io::provider::StreamType;
use serde::{Deserialize, Serialize};

pub trait EmitControlSpec: DataFraction {
    type State: DataFraction;
    type Action: DataFraction;

    fn stream_type() -> StreamType;

    fn apply(state: &mut Self::State, action: TimedEvent<Self::Action>);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitControlState<T: EmitControlSpec> {
    #[serde(bound = "")]
    pub spec: T,
    pub state: T::State,
}

impl<T: EmitControlSpec> EmitControlState<T> {
    pub fn new(spec: T, state: T::State) -> Self {
        Self { spec, state }
    }
}

impl<T: EmitControlSpec> Flow for EmitControlState<T> {
    type Action = EmitControlAction<T>;
    type Event = EmitControlEvent<T>;

    fn stream_type() -> StreamType {
        T::stream_type()
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            EmitControlEvent::ApplyAction { action } => {
                T::apply(&mut self.state, action);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmitControlAction<T: EmitControlSpec> {
    Emit { action: T::Action },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmitControlEvent<T: EmitControlSpec> {
    ApplyAction { action: TimedEvent<T::Action> },
}

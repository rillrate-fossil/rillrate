use rill_protocol::flow::core::{DataFraction, Flow};
use rill_protocol::io::provider::{Path, StreamType};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// It's like meta, but uses pulling for handling intensive load.
pub trait StatFlowSpec: DataFraction {
    type Stat: DataFraction + Default;
    type Delta: DataFraction;

    fn path(&self) -> Path;

    fn interval(&self) -> Duration;

    fn apply(stat: &mut Self::Stat, delta: Self::Delta);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatFlowState<T: StatFlowSpec> {
    pub stat: T::Stat,
}

#[allow(clippy::new_without_default)]
impl<T: StatFlowSpec> StatFlowState<T> {
    pub fn new() -> Self {
        Self {
            stat: T::Stat::default(),
        }
    }
}

impl<T: StatFlowSpec> Flow for StatFlowState<T> {
    type Action = StatFlowAction;
    type Event = StatFlowEvent<T>;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            StatFlowEvent::ApplyDelta { delta } => {
                T::apply(&mut self.stat, delta);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatFlowAction {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatFlowEvent<T: StatFlowSpec> {
    ApplyDelta { delta: T::Delta },
}

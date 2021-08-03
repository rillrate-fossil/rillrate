use rill_protocol::flow::core::{DataFraction, Flow};
use rill_protocol::io::provider::{Path, StreamType};
use serde::{Deserialize, Serialize};

pub trait MetaFlowSpec: DataFraction {
    type Meta: DataFraction;

    fn path() -> Path;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaFlowState<T: MetaFlowSpec> {
    pub meta: Option<T::Meta>,
}

#[allow(clippy::new_without_default)]
impl<T: MetaFlowSpec> MetaFlowState<T> {
    pub fn new() -> Self {
        Self { meta: None }
    }
}

impl<T: MetaFlowSpec> Flow for MetaFlowState<T> {
    type Action = MetaFlowAction;
    type Event = MetaFlowEvent<T>;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            MetaFlowEvent::SetMeta { meta } => {
                self.meta = Some(meta);
            }
            MetaFlowEvent::Clear => {
                self.meta.take();
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetaFlowAction {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetaFlowEvent<T: MetaFlowSpec> {
    SetMeta { meta: T::Meta },
    Clear,
}

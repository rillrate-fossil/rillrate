use crate::base::stat_flow::{StatFlowSpec, StatFlowState};
use crate::live_data::pulse::Range;
use rill_protocol::io::provider::StreamType;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GaugeSpec {
    // TODO: Move it outside...
    pub pull_ms: Option<u64>,
    pub range: Range,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GaugeStat {
    pub value: Option<f64>,
}

impl StatFlowSpec for GaugeSpec {
    type Stat = GaugeStat;
    type Delta = f64;

    fn stream_type() -> StreamType {
        StreamType::from(module_path!())
    }

    fn interval(&self) -> Option<Duration> {
        self.pull_ms.map(Duration::from_millis)
    }

    // TODO: Use `Spec` reference here to check the range
    fn apply(stat: &mut Self::Stat, delta: Self::Delta) {
        stat.value = Some(delta);
    }
}

pub type GaugeState = StatFlowState<GaugeSpec>;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeStat {
    pub value: Option<f64>,
    pub abs_min: f64,
    pub abs_max: f64,
}

impl Default for GaugeStat {
    fn default() -> Self {
        Self {
            value: None,
            abs_min: f64::MAX,
            abs_max: f64::MIN,
        }
    }
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
    fn apply(stat: &mut Self::Stat, value: Self::Delta) {
        stat.value = Some(value);
        if value < stat.abs_min {
            stat.abs_min = value;
        }
        if value > stat.abs_max {
            stat.abs_max = value;
        }
    }
}

pub type GaugeState = StatFlowState<GaugeSpec>;

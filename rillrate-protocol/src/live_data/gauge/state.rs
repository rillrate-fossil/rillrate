use crate::base::stat_flow::{StatFlowSpec, StatFlowState};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeSpec {
    // TODO: Move it outside...
    pub pull_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GaugeStat {
    pub value: f64,
}

impl StatFlowSpec for GaugeSpec {
    type Stat = GaugeStat;
    type Delta = f64;

    fn interval(&self) -> Option<Duration> {
        self.pull_ms.map(Duration::from_millis)
    }

    // TODO: Use `Spec` reference here to check the range
    fn apply(stat: &mut Self::Stat, delta: Self::Delta) {
        stat.value = delta;
    }
}

pub type GaugeState = StatFlowState<GaugeSpec>;

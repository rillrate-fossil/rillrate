use crate::base::frame_flow::{FrameFlowSpec, FrameFlowState};
use serde::{Deserialize, Serialize};

pub type PulseFrameState = FrameFlowState<PulseFrameSpec>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Range {
    pub min: Option<f32>,
    pub max: Option<f32>,
}

impl Range {
    pub fn new(mut min: f32, mut max: f32) -> Self {
        if min > max {
            std::mem::swap(&mut min, &mut max);
        }
        Self {
            min: Some(min),
            max: Some(max),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseFrameSpec {
    /// Retain interval in seconds.
    pub retain: u32,
    pub range: Range,
}

impl Default for PulseFrameSpec {
    fn default() -> Self {
        Self {
            retain: 30,
            range: Range::default(),
        }
    }
}

impl FrameFlowSpec for PulseFrameSpec {
    type Frame = f32;

    fn retain_secs(&self) -> u32 {
        self.retain
    }
}

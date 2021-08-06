use crate::base::frame_flow::{FrameFlowSpec, FrameFlowState};
use serde::{Deserialize, Serialize};

pub type PulseFrameState = FrameFlowState<PulseFrameSpec>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseFrameSpec {
    /// Retain interval in seconds.
    pub retain: u32,
}

impl FrameFlowSpec for PulseFrameSpec {
    type Frame = f32;

    fn retain_secs(&self) -> u32 {
        self.retain
    }
}

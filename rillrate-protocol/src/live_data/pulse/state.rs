use crate::base::frame_flow::{FrameFlowSpec, FrameFlowState};
use serde::{Deserialize, Serialize};

pub type PulseFrameState = FrameFlowState<PulseFrameSpec>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseFrameSpec {}

impl FrameFlowSpec for PulseFrameSpec {
    // TODO: Add range here
    type Info = ();
    type Frame = f32;

    fn retain_secs() -> u32 {
        31
    }
}

use crate::base_flow::frame_flow::{FrameFlowSpec, FrameFlowState};
use crate::live_flow::range::{Label, Range};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseSpec {
    /// Retain interval in seconds.
    // TODO: Make `retain` optional
    pub retain: u32,
    pub range: Range,
    pub label: Label,
}

impl Default for PulseSpec {
    fn default() -> Self {
        Self {
            retain: 30,
            range: Range::default(),
            label: Label::default(),
        }
    }
}

impl FrameFlowSpec for PulseSpec {
    type Frame = f64;

    fn retain_secs(&self) -> u32 {
        self.retain
    }
}

pub type PulseState = FrameFlowState<PulseSpec>;

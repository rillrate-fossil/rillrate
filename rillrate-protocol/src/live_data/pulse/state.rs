use crate::base::frame_flow::{FrameFlowSpec, FrameFlowState};
use serde::{Deserialize, Serialize};

// TODO: Move some parts here from the `rill-protocol::Range`
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

    pub fn min(min: f32) -> Self {
        Self {
            min: Some(min),
            max: None,
        }
    }

    pub fn max(max: f32) -> Self {
        Self {
            min: None,
            max: Some(max),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub caption: String,
    pub divisor: f32,
}

impl Default for Label {
    fn default() -> Self {
        Self {
            caption: String::new(),
            divisor: 1.0,
        }
    }
}

impl Label {
    pub fn new(caption: impl Into<String>, divisor: f32) -> Self {
        Self {
            caption: caption.into(),
            divisor,
        }
    }

    pub fn pct_100() -> Self {
        Self::new("%", 1.0)
    }

    pub fn pct_1() -> Self {
        Self::new("%", 1.0 / 100.0)
    }
}

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
    type Frame = f32;

    fn retain_secs(&self) -> u32 {
        self.retain
    }
}

pub type PulseState = FrameFlowState<PulseSpec>;

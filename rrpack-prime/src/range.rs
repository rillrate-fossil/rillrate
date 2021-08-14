use serde::{Deserialize, Serialize};

// TODO: Move some parts here from the `rill-protocol::Range`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Range {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl Range {
    pub fn new(mut min: f64, mut max: f64) -> Self {
        if min > max {
            std::mem::swap(&mut min, &mut max);
        }
        Self {
            min: Some(min),
            max: Some(max),
        }
    }

    pub fn min(min: f64) -> Self {
        Self {
            min: Some(min),
            max: None,
        }
    }

    pub fn max(max: f64) -> Self {
        Self {
            min: None,
            max: Some(max),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub caption: String,
    pub divisor: f64,
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
    pub fn new(caption: impl Into<String>, divisor: f64) -> Self {
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

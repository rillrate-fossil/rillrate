use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Bound {
    Auto,
    Accurate { value: f64, strict: bool },
}

impl Default for Bound {
    fn default() -> Self {
        Self::Auto
    }
}

// TODO: Move some parts here from the `rill-protocol::Range`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Range {
    pub min: Bound,
    pub max: Bound,
}

impl Range {
    pub fn new(mut min: f64, mut max: f64) -> Self {
        if min > max {
            std::mem::swap(&mut min, &mut max);
        }
        Self {
            min: Bound::Accurate {
                value: min,
                strict: true,
            },
            max: Bound::Accurate {
                value: max,
                strict: true,
            },
        }
    }

    pub fn min(min: f64) -> Self {
        Self {
            min: Bound::Accurate {
                value: min,
                strict: true,
            },
            max: Bound::Auto,
        }
    }

    pub fn max(max: f64) -> Self {
        Self {
            min: Bound::Auto,
            max: Bound::Accurate {
                value: max,
                strict: true,
            },
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

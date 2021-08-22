use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

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

impl Bound {
    pub fn from_options(value: Option<f64>, loose: Option<bool>) -> Self {
        match value {
            Some(value) => {
                let strict = !loose.unwrap_or_default();
                Self::Accurate { value, strict }
            }
            None => Self::Auto,
        }
    }

    pub fn auto() -> Self {
        Self::Auto
    }

    pub fn strict(value: impl Into<f64>) -> Self {
        Self::Accurate {
            value: value.into(),
            strict: true,
        }
    }

    pub fn loose(value: impl Into<f64>) -> Self {
        Self::Accurate {
            value: value.into(),
            strict: false,
        }
    }

    pub fn min(&self, active_min: f64) -> f64 {
        self.clamp(active_min, Ordering::Less)
    }

    pub fn max(&self, active_max: f64) -> f64 {
        self.clamp(active_max, Ordering::Greater)
    }

    fn clamp(&self, active: f64, ordering: Ordering) -> f64 {
        match *self {
            Self::Auto => active,
            Self::Accurate { value, strict } => {
                if active.partial_cmp(&value) == Some(ordering) {
                    if strict {
                        value
                    } else {
                        active
                    }
                } else {
                    value
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)] // Consts are used only!
mod tests {
    use super::*;

    #[test]
    fn test_bound_min() {
        let auto = Bound::Auto;
        assert_eq!(auto.min(10.0), 10.0);
        assert_eq!(auto.min(-5.0), -5.0);
        let strict = Bound::Accurate {
            value: 0.0,
            strict: true,
        };
        assert_eq!(strict.min(10.0), 0.0);
        assert_eq!(strict.min(0.0), 0.0);
        assert_eq!(strict.min(-5.0), 0.0);
        let loose = Bound::Accurate {
            value: 0.0,
            strict: false,
        };
        assert_eq!(loose.min(10.0), 0.0);
        assert_eq!(loose.min(0.0), 0.0);
        assert_eq!(loose.min(-5.0), -5.0);
    }

    #[test]
    fn test_bound_max() {
        let auto = Bound::Auto;
        assert_eq!(auto.max(120.0), 120.0);
        assert_eq!(auto.max(90.0), 90.0);
        let strict = Bound::Accurate {
            value: 100.0,
            strict: true,
        };
        assert_eq!(strict.max(120.0), 100.0);
        assert_eq!(strict.max(100.0), 100.0);
        assert_eq!(strict.max(90.0), 100.0);
        let loose = Bound::Accurate {
            value: 100.0,
            strict: false,
        };
        assert_eq!(loose.max(120.0), 120.0);
        assert_eq!(loose.max(100.0), 100.0);
        assert_eq!(loose.max(90.0), 100.0);
    }
}

// TODO: Move some parts here from the `rill-protocol::Range`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Range {
    pub min: Bound,
    pub max: Bound,
}

impl Range {
    // TODO: Remove it
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

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Range {
    min: OrderedFloat<f64>,
    max: OrderedFloat<f64>,
}

impl Range {
    pub fn new(mut min: f64, mut max: f64) -> Self {
        if min > max {
            std::mem::swap(&mut min, &mut max);
        }
        Self {
            min: OrderedFloat::from(min),
            max: OrderedFloat::from(max),
        }
    }

    pub fn min(&self) -> f64 {
        *self.min
    }

    pub fn max(&self) -> f64 {
        *self.max
    }

    pub fn diff(&self) -> f64 {
        *(self.max - self.min)
    }

    pub fn clamp(&self, value: &mut f64) {
        if *value < *self.min {
            *value = *self.min
        } else if *value > *self.max {
            *value = *self.max
        }
    }

    pub fn pct(&self, value: f64) -> Pct {
        Pct::from_range(value, self)
    }
}

impl From<(f64, f64)> for Range {
    fn from((min, max): (f64, f64)) -> Self {
        Range::new(min, max)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct Pct(f64);

impl Pct {
    pub fn from_value(mut value: f64) -> Self {
        // TODO: Use `clamp` here.
        if value < 0.0 {
            value = 0.0;
        } else if value > 1.0 {
            value = 1.0;
        }
        Self(value)
    }

    pub fn from_div(value: f64, total: f64) -> Self {
        let pct = {
            if total == 0.0 {
                0.0
            } else {
                value / total
            }
        };
        Pct::from_value(pct)
    }

    pub fn from_range(value: f64, range: &Range) -> Self {
        let value = value - range.min();
        let diff = range.diff();
        Pct::from_div(value, diff)
    }

    pub fn to_cent(&self) -> f64 {
        (self.0 * 100.0).round()
    }
}

impl Deref for Pct {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

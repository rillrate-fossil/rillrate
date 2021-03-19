use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

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
        Self { min: OrderedFloat::from(min), max: OrderedFloat::from(max) }
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
}

impl From<(f64, f64)> for Range {
    fn from((min, max): (f64, f64)) -> Self {
        Range::new(min, max)
    }
}

use derive_more::{From, Into};
use std::f64::consts;

#[derive(Debug, Clone, PartialEq)]
pub struct Range {
    start: f64,
    stop: f64,
    min: f64,
    max: f64,
    diff: f64,
    reverse: bool,
}

impl Range {
    pub fn new(start: f64, stop: f64) -> Self {
        if start <= stop {
            Self {
                start,
                stop,
                min: start,
                max: stop,
                diff: stop - start,
                reverse: false,
            }
        } else {
            Self {
                start,
                stop,
                min: stop,
                max: start,
                diff: start - stop,
                reverse: true,
            }
        }
    }

    pub fn min(&self) -> f64 {
        self.min
    }

    pub fn max(&self) -> f64 {
        self.max
    }

    /*
    pub fn start(&self) -> f64 {
        self.start
    }

    pub fn stop(&self) -> f64 {
        self.stop
    }

    pub fn diff(&self) -> f64 {
        self.diff
    }

    pub fn reverse(&self) -> bool {
        self.reverse
    }
    */

    pub fn with_padding(&mut self, mut pad: f64) {
        if self.reverse {
            pad = -pad;
        }
        *self = Self::new(self.start + pad, self.stop - pad);
    }

    // TODO: Fix. It doesn't work on `0`
    // TODO: Implement `nice`?
    pub fn spread(&mut self, mut spread: f64) {
        if self.reverse {
            spread = -spread;
        }
        *self = Self::new(self.start * (1.0 - spread), self.stop * (1.0 + spread));
    }

    pub fn is_flat(&self) -> bool {
        self.diff == 0.0
    }
}

impl From<(f64, f64)> for Range {
    fn from((min, max): (f64, f64)) -> Self {
        Self::new(min, max)
    }
}

pub struct LinearScale {
    domain: Range,
    range: Range,
}

impl LinearScale {
    pub fn new(domain: Range, range: Range) -> Self {
        Self { domain, range }
    }

    pub fn rescale(&self, value: f64) -> f64 {
        self.range.interpolate(self.domain.normalize(value))
    }
}

impl Range {
    fn tick_increment(&self, count: f64) -> f64 {
        let start = self.min;
        let stop = self.max;

        let e10 = 50_f64.sqrt();
        let e5 = 10_f64.sqrt();
        let e2 = 2_f64.sqrt();

        let step = (stop - start) / count.max(0.0);
        let power = (step.ln() / consts::LN_10).floor();
        let error = step / 10_f64.powf(power);

        let factor;
        if error >= e10 {
            factor = 10.0;
        } else if error >= e5 {
            factor = 5.0;
        } else if error >= e2 {
            factor = 2.0;
        } else {
            factor = 1.0;
        }

        if power >= 0.0 {
            factor * 10_f64.powf(power)
        } else {
            -(10_f64.powf(-power)) / factor
        }
    }

    pub fn ticks(&self, count: u16) -> Vec<f64> {
        let start = self.min;
        let stop = self.max;
        let mut ticks = Vec::with_capacity(count as usize);
        if self.diff == 0.0 && count > 0 {
            ticks.push(start);
        } else {
            let step = self.tick_increment(count as f64);
            if step.is_finite() {
                if step > 0.0 {
                    let mut r_start = (start / step).round();
                    let mut r_stop = (stop / step).round();
                    if r_start * step < start {
                        r_start += 1.0;
                    }
                    if r_stop * step > stop {
                        r_stop -= 1.0;
                    }
                    let n = (r_stop - r_start + 1.0) as usize;
                    let iter = (0..n).map(|i| (r_start + i as f64) * step);
                    if !self.reverse {
                        ticks.extend(iter);
                    } else {
                        ticks.extend(iter.rev());
                    }
                } else if step < 0.0 {
                    let step = -step;
                    let mut r_start = (start * step).round();
                    let mut r_stop = (stop * step).round();
                    if r_start / step < start {
                        r_start += 1.0;
                    }
                    if r_stop / step > stop {
                        r_stop -= 1.0;
                    }
                    let n = (r_stop - r_start + 1.0) as usize;
                    let iter = (0..n).map(|i| (r_start + i as f64) / step);
                    if !self.reverse {
                        ticks.extend(iter);
                    } else {
                        ticks.extend(iter.rev());
                    }
                } else {
                    // and skip if `step == 0.0`
                }
            }
        }
        ticks
    }

    fn interpolate(&self, norm: Norm) -> f64 {
        let value = f64::from(norm);
        if !self.reverse {
            self.min * (1.0 - value) + self.max * value
        } else {
            self.min * value + self.max * (1.0 - value)
        }
    }

    fn normalize(&self, value: f64) -> Norm {
        if !self.reverse {
            ((value - self.min) / self.diff).into()
        } else {
            ((self.max - value) / self.diff).into()
        }
    }
}

// Normalized value (in range [0;1])
#[derive(From, Into, Debug, PartialEq, PartialOrd)]
struct Norm(pub f64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_increment() {
        let range = Range::new(0.03310319276564422, 0.9859442826901874);
        assert_eq!(range.tick_increment(5.0), -5.0);

        let range = Range::new(0.12, 500.0);
        assert_eq!(range.tick_increment(29.0), 20.0);
    }

    #[test]
    fn test_range_ticks() {
        let range = Range::new(1.0, 10.0);
        assert_eq!(range.ticks(5), vec![2.0, 4.0, 6.0, 8.0, 10.0]);

        let range = Range::new(0.03310319276564422, 0.9859442826901874);
        assert_eq!(range.ticks(5), vec![0.2, 0.4, 0.6, 0.8]);
    }

    #[test]
    fn test_linear_rescale() {
        let domain = Range::new(10.0, 20.0);
        assert_eq!(domain.normalize(11.0), Norm(0.1));
        let range = Range::new(0.0, 600.0);
        let linear = LinearScale::new(domain, range);
        assert_eq!(linear.rescale(11.0), 60.0);
    }
}

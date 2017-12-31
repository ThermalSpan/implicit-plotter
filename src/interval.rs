
use function::Function;
use itertools::Itertools;
use std::collections::HashMap;
use std::f32;

#[derive(Copy, Clone, Debug)]
pub struct Interval {
    pub min: f32,
    pub max: f32,
}

impl Interval {
    pub fn add(
        &self,
        other: &Interval,
    ) -> Vec<Interval> {
        vec![
            Interval {
                min: self.min + other.min,
                max: self.max + other.max,
            },
        ]
    }

    pub fn sub(
        &self,
        other: &Interval,
    ) -> Vec<Interval> {
        vec![
            Interval {
                min: self.min - other.min,
                max: self.max - other.max,
            },
        ]
    }

    pub fn mul(
        &self,
        other: &Interval,
    ) -> Vec<Interval> {
        let minmax = [self.min, self.max]
            .iter()
            .cartesian_product(&[other.min, other.max])
            .map(|(min, max)| min * max)
            .minmax()
            .into_option()
            .unwrap();

        vec![
            Interval {
                min: minmax.0,
                max: minmax.1,
            },
        ]
    }

    pub fn div(
        &self,
        other: &Interval,
    ) -> Vec<Interval> {
        let inverse = match (other.min, other.max) {
            (_, _) if !other.contains_zero() => {
                Interval {
                    min: 1.0 / other.min,
                    max: 1.0 / other.max,
                }
            },
            (min, max) if max == 0.0 => {
                Interval {
                    min: -f32::INFINITY,
                    max: 1.0 / min,
                }
            },
            (min, max) if min == 0.0 => {
                Interval {
                    min: 1.0 / max,
                    max: f32::INFINITY,
                }
            },
            (min, max) => {
                Interval {
                    min: -f32::INFINITY,
                    max: f32::INFINITY,
                }
            },
        };

        self.mul(&inverse)
    }

    pub fn exp(
        &self,
        power: &Interval,
    ) -> Vec<Interval> {
        // First we need to eliminate invalid exponentiation calls
        // That means no negative bases
        if self.max < 0.0 {
            return Vec::new();
        }

        if self.min < 0.0 {
            // TODO: we should explore the consequences of this
            // For now, only allow exponentiation on defined ranges
            return Interval {
                min: 0.0,
                max: self.max,
            }.exp(power);
        }

        // TODO: we need to improve the logic here to isolate powers in [-1, 1] and
        // make sure they
        // are split into the two possible options
        let minmax = [self.min, self.max]
            .iter()
            .cartesian_product(&[power.min, power.max])
            .map(|(base, power)| base.powf(*power))
            .minmax()
            .into_option()
            .unwrap();

        vec![
            Interval {
                min: minmax.0,
                max: minmax.1,
            },
        ]
    }

    pub fn contains_zero(&self) -> bool {
        self.min < 0.0 && self.max > 0.0
    }
}

pub fn permute_intervals<A, F>(
    node1: &Box<A>,
    node2: &Box<A>,
    bindings: &HashMap<char, Interval>,
    op: F,
) -> Vec<Interval>
where
    F: FnMut((&Interval, &Interval)) -> Vec<Interval>,
    A: Function, {
    let n1_i = node1.evaluate_interval(&bindings);
    let n2_i = node2.evaluate_interval(&bindings);

    n1_i.iter().cartesian_product(&n2_i).map(op).concat()
}

pub fn contains_zero(intervals: &[Interval]) -> bool {
    for interval in intervals {
        if interval.contains_zero() {
            return true;
        }
    }
    false
}


use interval::Interval;
use std::collections::HashMap;

pub trait Function {
    fn evaluate(
        &self,
        x: f32,
        y: f32,
        z: f32,
    ) -> f32;

    fn evaluate_interval(
        &self,
        bindings: &HashMap<char, Interval>,
    ) -> Vec<Interval>;
}

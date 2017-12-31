use gen::Function;
use itertools::Itertools;
use std::collections::HashMap;
use std::f32;

#[derive(Copy, Clone, Debug)]
pub struct Interval {
    min: f32,
    max: f32,
}

impl Interval {
    fn add(
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

    fn sub(
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

    fn mul(
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

    fn div(
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

    fn exp(
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

    fn contains_zero(&self) -> bool {
        self.min < 0.0 && self.max > 0.0
    }
}

fn permute_intervals<F>(
    node1: &Node,
    node2: &Node,
    bindings: &HashMap<char, Interval>,
    op: F,
) -> Vec<Interval>
where
    F: FnMut((&Interval, &Interval)) -> Vec<Interval>, {
    let n1_i = node1.evaluate_interval(&bindings);
    let n2_i = node2.evaluate_interval(&bindings);

    n1_i.iter().cartesian_product(&n2_i).map(op).concat()
}

fn contains_zero(intervals: &[Interval]) -> bool {
    for interval in intervals {
        if interval.contains_zero() {
            return true;
        }
    }
    false
}

#[derive(PartialEq, Debug)]
pub enum Node {
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
    Exp(Box<Node>, Box<Node>),
    Variable(char),
    Constant(f32),
}

impl Node {
    pub fn evaluate(
        &self,
        bindings: &HashMap<char, f32>,
    ) -> f32 {
        match *self {
            Node::Add(ref n1, ref n2) => n1.evaluate(&bindings) + n2.evaluate(&bindings),
            Node::Sub(ref n1, ref n2) => n1.evaluate(&bindings) - n2.evaluate(&bindings),
            Node::Mul(ref n1, ref n2) => n1.evaluate(&bindings) * n2.evaluate(&bindings),
            Node::Div(ref n1, ref n2) => n1.evaluate(&bindings) / n2.evaluate(&bindings),
            Node::Exp(ref n1, ref n2) => n1.evaluate(&bindings).powf(n2.evaluate(&bindings)),
            Node::Constant(c) => c,
            Node::Variable(v) => bindings.get(&v).unwrap().clone(),
        }
    }

    pub fn evaluate_interval(
        &self,
        bindings: &HashMap<char, Interval>,
    ) -> Vec<Interval> {
        match *self {
            Node::Add(ref n1, ref n2) => {
                permute_intervals(n1, n2, &bindings, |(interval1, interval2)| {
                    interval1.add(interval2)
                })
            },
            Node::Sub(ref n1, ref n2) => {
                permute_intervals(n1, n2, &bindings, |(interval1, interval2)| {
                    interval1.sub(interval2)
                })
            },
            Node::Mul(ref n1, ref n2) => {
                permute_intervals(n1, n2, &bindings, |(interval1, interval2)| {
                    interval1.mul(interval2)
                })
            },
            Node::Exp(ref n1, ref n2) => {
                permute_intervals(n1, n2, &bindings, |(interval1, interval2)| {
                    interval1.exp(interval2)
                })
            },
            Node::Div(ref n1, ref n2) => {
                permute_intervals(n1, n2, &bindings, |(interval1, interval2)| {
                    interval1.div(interval2)
                })
            },
            Node::Constant(c) => vec![Interval { min: c, max: c }],
            Node::Variable(v) => vec![bindings.get(&v).unwrap().clone()], 
        }
    }
}

impl Function for Node {
    fn evaluate(
        &self,
        x: f32,
        y: f32,
        z: f32,
    ) -> f32 {
        let mut bindings = HashMap::new();
        bindings.insert('x', x);
        bindings.insert('y', y);
        bindings.insert('z', z);

        self.evaluate(&bindings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::parse_expression;
    use std::collections::HashMap;
    #[macro_use]
    use util;

    #[test]
    fn test_function_evaluate() {
        let mut input: Vec<char>;
        let mut root;
        let mut bindings = HashMap::new();
        bindings.insert('x', 1.13);
        bindings.insert('y', 4.232);
        bindings.insert('z', 2.0939);

        input = "x".chars().collect();
        root = parse_expression(&input, 0).unwrap();
        assert_similiar!(root.evaluate(&bindings), 1.13);

        input = "x + y ^ z".chars().collect();
        root = parse_expression(&input, 0).unwrap();
        assert_similiar!(root.evaluate(&bindings), 21.6380);

        input = "x + y - z".chars().collect();
        root = parse_expression(&input, 0).unwrap();
        assert_similiar!(root.evaluate(&bindings), 3.2681);

        input = "x + y - z / x - y + z".chars().collect();
        root = parse_expression(&input, 0).unwrap();
        assert_similiar!(root.evaluate(&bindings), 1.3709);

        input = "x + y - (z / x) - y + z".chars().collect();
        root = parse_expression(&input, 0).unwrap();
        assert_similiar!(root.evaluate(&bindings), 1.3709);

        input = "3.2 ^ (0.01 / 8) + (4.0 * 3 + 2 - 3^7 - (4)) / z ^ 2"
            .chars()
            .collect();
        root = parse_expression(&input, 0).unwrap();
        assert_similiar!(root.evaluate(&bindings), -495.5297);
    }

    #[test]
    fn test_function_inteval() {
        let mut input: Vec<char>;
        let mut root;
        let mut result;
        let mut bindings = HashMap::new();
        bindings.insert(
            'x',
            Interval {
                min: 0.01,
                max: 3.1,
            },
        );
        bindings.insert(
            'y',
            Interval {
                min: -5.0,
                max: 5.0,
            },
        );
        bindings.insert(
            'z',
            Interval {
                min: -3.0,
                max: -1.0,
            },
        );

        input = "x".chars().collect();
        root = parse_expression(&input, 0).unwrap();
        result = root.evaluate_interval(&bindings);
        assert_eq!(result.len(), 1);
        assert_similiar!(result[0].min, 0.01);
        assert_similiar!(result[0].max, 3.1);

        input = "x+y".chars().collect();
        root = parse_expression(&input, 0).unwrap();
        result = root.evaluate_interval(&bindings);
        assert_eq!(result.len(), 1);
        assert_similiar!(result[0].min, -4.99);
        assert_similiar!(result[0].max, 8.1);

        input = "x*y".chars().collect();
        root = parse_expression(&input, 0).unwrap();
        result = root.evaluate_interval(&bindings);
        assert_eq!(result.len(), 1);
        assert_similiar!(result[0].min, -15.5);
        assert_similiar!(result[0].max, 15.5);

        // TDOD add more tests once behaivor settles
    }
}

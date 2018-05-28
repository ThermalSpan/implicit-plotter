use function::*;
use interval::Interval;
use itertools::Itertools;
use std::collections::HashMap;
use std::io::Write;

#[derive(Copy, Clone)]
struct BoundingBox {
    x: Interval,
    y: Interval,
    z: Interval,
}

impl BoundingBox {
    fn split(&self) -> Vec<BoundingBox> {
        let x_is = self.x.split();
        let y_is = self.y.split();
        let z_is = self.z.split();

        x_is.iter()
            .cartesian_product(y_is.iter())
            .cartesian_product(z_is.iter())
            .map(|((x, y), z)| {
                BoundingBox {
                    x: x.clone(),
                    y: y.clone(),
                    z: z.clone(),
                }
            })
            .collect()
    }
}

struct MNode {
    intervals: Vec<Interval>,
    bb: BoundingBox,
    children: Option<Vec<MNode>>,
}

impl MNode {
    fn split<F: Function>(
        &mut self,
        f: &Box<F>,
    ) {
        let children = self.bb
            .split()
            .iter()
            .map(|bb| {
                let mut bindings = HashMap::new();
                bindings.insert('x', bb.x);
                bindings.insert('y', bb.x);
                bindings.insert('z', bb.x);
                let intervals = f.evaluate_interval(&bindings);
                MNode {
                    bb: bb.clone(),
                    intervals: intervals,
                    children: None,
                }
            })
            .collect();

        self.children = Some(children);
    }

    pub fn write_as_plot<W: Write>(
        &self,
        writer: &mut W,
    ) {
        let bb = self.bb;
        let lines = [
            // Bottom face
            (bb.x.min, bb.y.min, bb.z.min, bb.x.max, bb.y.min, bb.z.min),

            (bb.x.max, bb.y.min, bb.z.min, bb.x.max, bb.y.max, bb.z.min),

            (bb.x.max, bb.y.max, bb.z.min, bb.x.min, bb.y.max, bb.z.min),

            (bb.x.min, bb.y.max, bb.z.min, bb.x.min, bb.y.min, bb.z.min),

            // Top Face,
            (bb.x.min, bb.y.min, bb.z.max, bb.x.max, bb.y.min, bb.z.max),

            (bb.x.max, bb.y.min, bb.z.max, bb.x.max, bb.y.max, bb.z.max),

            (bb.x.max, bb.y.max, bb.z.max, bb.x.min, bb.y.max, bb.z.max),

            (bb.x.min, bb.y.max, bb.z.max, bb.x.min, bb.y.min, bb.z.max),

            // Legs
            (bb.x.min, bb.y.min, bb.z.min, bb.x.min, bb.y.min, bb.z.max),

            (bb.x.min, bb.y.max, bb.z.min, bb.x.min, bb.y.max, bb.z.max),

            (bb.x.max, bb.y.max, bb.z.min, bb.x.max, bb.y.max, bb.z.max),

            (bb.x.max, bb.y.min, bb.z.min, bb.x.max, bb.y.min, bb.z.max),
        ];

        for l in &lines {
            writeln!(
                writer,
                "<line> {} {} {} {} {} {} </line>",
                l.0,
                l.1,
                l.2,
                l.3,
                l.4,
                l.5
            );
        }

        if let Some(ref children) = self.children {
            for c in children {
                c.write_as_plot(writer);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::*;
    use std::fs::File;
    /*
    #[test]
    fn write_test() {
        let unit_i = Interval { min: 0.0, max: 1.0 };

        let unit_b = BoundingBox {
            x: unit_i.clone(),
            y: unit_i.clone(),
            z: unit_i.clone(),
        };

        let mut n = MNode {
            intervals: Vec::new(),
            bb: unit_b,
            children: None,
        };

        let f = Box::new(ConstFunction { c: 1.0 });
        n.split(&f);
        n.children.unwrap()[0].split(&f);

        let mut file = File::create("/Users/russell/bb.txt").unwrap();
        n.write_as_plot(&mut file);
    }
*/
    #[test]
    fn write_test_1() {
        let unit_i = Interval { min: 0.0, max: 1.0 };

        let unit_b = BoundingBox {
            x: unit_i.clone(),
            y: unit_i.clone(),
            z: unit_i.clone(),
        };

        let mut n = MNode {
            intervals: Vec::new(),
            bb: unit_b,
            children: None,
        };

        let input: Vec<char> = "x^2 + y^2 + z^2 - 0.99".chars().collect();
        let f = parse_expression(&input, 0).unwrap();
        n.split(&f);
        // n.children.unwrap().get_mut(0).unwrap().split(&f);

        let mut file = File::create("/Users/russell/bb1.txt").unwrap();
        n.write_as_plot(&mut file);
    }

}

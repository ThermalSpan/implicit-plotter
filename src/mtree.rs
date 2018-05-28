use function::*;
use interval::Interval;
use itertools::Itertools;
use std::collections::HashMap;
use std::io::Write;

use geoprim::*;

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

    pub fn add_to_plot(&self, plot: &mut Plot ) {
        let bb = self.bb;

        // Build up the outline of a cube
        //
        // 1.) Make a point buffer with all the corners
        let mut points = Vec::new();
        for x in vec![bb.x.min, bb.x.max] {
            for y in vec![bb.y.min, bb.y.max] {
                for z in vec![bb.z.min, bb.z.max] {
                    points.push(Point::new(x, y, z));
                }
            }
        }

        // 2.) make a line buffer with appropriate endpoints
        let index_pairs = vec![
            (0, 1),
            (1, 3),
            (3, 2),
            (2, 0),
            (4, 5),
            (5, 7),
            (7, 6),
            (6, 4),
            (0, 4),
            (1, 5),
            (3, 7),
            (2, 6)
        ];
        for (p1, p2) in index_pairs {
            plot.add_line(LineSegment::new(points[p1], points[p2]));
        }

        if let Some(ref children) = self.children {
            for c in children {
                c.add_to_plot(plot);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::*;
    use std::fs::File;
    use serde_json;
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
        //n.children.unwrap().get_mut(0).unwrap().split(&f);
    
        let mut plot = Plot::new();
        n.add_to_plot(&mut plot);

        let mut file = File::create("/Users/russell/bb1.txt").unwrap();
        serde_json::to_writer_pretty(&mut file, &plot);
    }
}

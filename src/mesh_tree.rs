use function::*;
use interval::Interval;
use cgmath::Vector3;
use interval::contains_zero;
use itertools::Itertools;
use std::iter::FromIterator;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::iter::Iterator;
use geoprim::*;
use key;
use key::Key;

pub struct Geometry {
    pub vertices: Vec<Vector3<f32>>,
    pub triangles: Vec<u32>,
    pub lines: Vec<u32>
}


#[derive(Debug, Copy, Clone)]
pub struct BoundingBox {
    pub x: Interval,
    pub y: Interval,
    pub z: Interval,
}

impl BoundingBox {
    pub fn split(&self) -> Vec<BoundingBox> {
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

    pub fn contains_root<F: Function>(&self, f: &Box<F>) -> bool {
         let mut bindings = HashMap::new();
        bindings.insert('x', self.x);
        bindings.insert('y', self.y);
        bindings.insert('z', self.z);
        let intervals = f.evaluate_interval(&bindings);
        contains_zero(&intervals)
    }

    pub fn center(&self) -> Vector3<f32> {
        Vector3::new(
            self.x.middle(),
            self.y.middle(),
            self.z.middle()
        )
    }

    pub fn clamp_vector(&self, v: &mut Vector3<f32>) {
        v.x = self.x.clamp_value(v.x);
        v.y = self.y.clamp_value(v.y);
        v.z = self.z.clamp_value(v.z);
    }

    pub fn add_to_plot(&self, plot: &mut Plot ) {
        // Build up the outline of a cube
        //
        // 1.) Make a point buffer with all the corners
        let mut points = Vec::new();
        for x in vec![self.x.min, self.x.max] {
            for y in vec![self.y.min, self.y.max] {
                for z in vec![self.z.min, self.z.max] {
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
    }
}

pub struct MeshTree<K: key::Key, F: Function> {
    function: Box<F>,
    pub level: u32,
    solution_map: HashMap<K, BoundingBox>,
    vertex_map: HashMap<K, Vector3<f32>>,
    edge_set: HashSet<(K, K)>,
    triangles: Vec<(K, K, K)>, 
}


impl  <F: Function> MeshTree<key::MortonKey, F> {
    pub fn new(f: Box<F>, bb: BoundingBox) -> MeshTree<key::MortonKey, F> {
        let mut result = MeshTree {
            function: f,
            level: 0,
            edge_set: HashSet::new(),
            solution_map: HashMap::new(),
            vertex_map: HashMap::new(),
            triangles: Vec::new(),
        };

        let root_key = key::MortonKey::root_key();
        if bb.contains_root(&result.function) {
            result.solution_map.insert(root_key, bb);
        }

        result
    }

    pub fn next_level(&mut self) {
        self.vertex_map.clear();
        self.edge_set.clear();
        self.triangles.clear();
        self.level += 1;

        let mut new_solution_map = HashMap::new();

        for (key, bb) in &self.solution_map {
            let child_keys: Vec<key::MortonKey> = (0..8u64).map(|i| key.child_key(i)).collect();
            let child_bb = bb.split();

            for i in 0..8 {
                let child_key = child_keys[i];
                let child_bb = child_bb[i];
                if child_bb.contains_root(&self.function) {
                    new_solution_map.insert(child_key, child_bb);
                }
            }
        }

        self.solution_map = new_solution_map;
    }

    pub fn generate_vertex_map(&mut self) {
        self.vertex_map.clear();
        for (key, bb) in &self.solution_map {
            self.vertex_map.insert(key.clone(), bb.center());
        }
    }

    pub fn generate_edge_set(&mut self) {
        let key_set: HashSet<key::MortonKey> = HashSet::from_iter(self.solution_map.keys().map(|k| k.clone()));
        for key in &key_set {
            key.neighbors().iter()
                .filter(|&n_k| n_k > &key && key_set.contains(n_k))
                .for_each(|n_k| {self.edge_set.insert((key.clone(), n_k.clone()));})
        }
    }

    pub fn relax_vertices(&mut self) {
        let mut new_vertex_map = HashMap::new();
        for (key, vertex) in &self.vertex_map {
            let neighbors: Vec<key::MortonKey> = key.neighbors();

            let mut sum = Vector3::new(0.0, 0.0, 0.0);
            let mut count = 0;
            for neighbor_key in neighbors {
                if let Some(neighbor) = self.vertex_map.get(&neighbor_key) {
                    count += 1;
                    sum = sum + neighbor; 
                }
            }

            if count == 0 {
                new_vertex_map.insert(key.clone(), vertex.clone());
                continue;
            }

            sum /= count as f32;

            let mut new_v = vertex + 0.5 * (sum - vertex);
            let bb = self.solution_map.get(&key).unwrap();
            bb.clamp_vector(&mut new_v);

            new_vertex_map.insert(key.clone(), new_v);
        }

        self.vertex_map = new_vertex_map;
    }

    pub fn add_to_plot(&self, add_bb: bool, add_vertices: bool, add_edges: bool, plot: &mut Plot ) {
        if add_bb {
            for bb in self.solution_map.values() {
                bb.add_to_plot(plot);
            }
        }

        if add_vertices {
            for vertex in self.vertex_map.values() {
                plot.add_point(Point { x: vertex.x, y: vertex.y, z: vertex.z });
            }
        }

        if add_edges {
            for (key1, key2) in &self.edge_set {
                let c1 = &self.vertex_map.get(key1).unwrap();
                let c2 = &self.vertex_map.get(key2).unwrap();

                let p1 = Point {
                    x: c1.x, y: c1.y, z: c2.z
                };

                let p2 = Point {
                    x: c2.x, y: c2.y, z: c2.z
                };

                plot.add_line(LineSegment::new(p1, p2));
            }
        }
    }
}


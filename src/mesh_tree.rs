use function::*;
use interval::Interval;
use interval::contains_zero;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use geoprim::*;
use key;

pub struct Vertex {
    position: [f32; 3]
}

pub struct Geometry {
    pub vertices: Vec<Vertex>,
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

    pub fn contains_root<F: Function>(&self, f: &F) -> bool {
         let mut bindings = HashMap::new();
        bindings.insert('x', self.x);
        bindings.insert('y', self.y);
        bindings.insert('z', self.z);
        let intervals = f.evaluate_interval(&bindings);
        contains_zero(&intervals)
    }

    pub fn center(&self) -> Vertex {
        Vertex {
            position: [
                self.x.middle(),
                self.y.middle(),
                self.z.middle()
            ]
        }
    }
}

pub struct MeshTree<K: key::Key, F: Function> {
    function: F,
    level: u32,
    solution_map: HashMap<K, BoundingBox>,
    vertex_map: HashMap<K, Vertex>,
    line_set: HashSet<(K, K)>,
    triangles: Vec<(K, K, K)>, 
}


impl  <K: key::Key, F: Function> MeshTree<K, F> {
    fn new(f: F, bb: BoundingBox) -> MeshTree<K, F> {
        let mut solution_map = HashMap::new();
        let mut vertex_map = HashMap::new();
        let root_key = K::root_key();

        if bb.contains_root(&f) {
            vertex_map.insert(root_key, bb.center());
            solution_map.insert(root_key, bb);
        }
        
        MeshTree {
            function: f,
            level: 0,
            solution_map,
            vertex_map,
            line_set: HashSet::new(),
            triangles: Vec::new(),
        }
    }

    fn next_level(&mut self) {
        self.vertex_map.clear();
        self.line_set.clear();
        self.triangles.clear();

        let new_solution_map = HashMap::new();

        for (key, bb) in &self.solution_map {
            
        }

        self.solution_map = new_solution_map;
    }
}



extern crate itertools;
extern crate geoprim;
extern crate serde_json;
extern crate cgmath;

#[macro_use]
mod util;

pub mod interval;
pub mod function;
pub mod function_ir;
pub mod gen;
pub mod parser;
pub mod parser_error;
//pub mod mtree;
pub mod mesh_tree;
pub mod key;


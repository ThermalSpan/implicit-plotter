use std::collections::HashMap;

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
}

pub struct NodeFunction {}

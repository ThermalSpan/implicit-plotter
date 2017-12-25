#[derive(PartialEq, Debug)]
pub enum Node {
    Mul(Box<Node>, Box<Node>),
    Exp(Box<Node>, Box<Node>),
    Variable(char),
    Constant(f32),
}

pub struct NodeFunction {

}

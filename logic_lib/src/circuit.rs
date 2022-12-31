use std::rc::{Rc, Weak};

use petgraph::{Directed, Graph};

use crate::bitref::BitRef;
use crate::component::Component;
/* Basically a graph of gates or other circuits */
pub struct Circuit {
    graph: Graph<Component, Rc<BitRef>, Directed>,
    inputs: Vec<Rc<BitRef>>,
    outputs: Vec<Weak<BitRef>>,
}

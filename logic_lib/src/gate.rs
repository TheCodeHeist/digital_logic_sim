use std::rc::{Rc, Weak};

use crate::bitref::BitRef;
use crate::common::*;

pub struct LogicGate {
    facing: Direction,
    num_bits: usize,
    num_inputs: usize,
    gate_size: GateSize,
    label: String,
    inputs: Vec<Rc<BitRef>>,
    output: Weak<BitRef>,
    negate: Vec<bool>,
}

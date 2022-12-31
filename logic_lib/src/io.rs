use std::rc::Rc;

use crate::{bitref::BitRef, common::Direction};

pub struct Pin {
    facing: Direction,
    output: bool,
    num_bits: usize,
    data: Rc<BitRef>,
}

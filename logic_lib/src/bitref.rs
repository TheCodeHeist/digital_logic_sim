use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

#[derive(PartialEq)]
pub enum State {
    Unknown,
    DontCare,
    Bits(Vec<bool>),
}

/* BitRef basically functions like wires. It has a mutable state buffer, which output pins will write to and input pins will read. */
pub struct BitRef {
    pub num_bits: usize,
    data: RefCell<State>,
}

impl BitRef {
    fn new(bits: usize) -> BitRef {
        BitRef {
            num_bits: bits,
            data: RefCell::new(State::Unknown),
        }
    }

    fn write(&mut self, state: State) -> State {
        self.data.replace(state)
    }

    fn read(&self) -> Ref<State> {
        self.data.borrow()
    }
}

#[derive(Eq, PartialEq, PartialOrd, Ord)]
pub enum Op {
    NOT,
    AND,
    OR,
    NAND,
    NOR,
    XOR,
    XNOR,
}

pub fn op(inputs: &Vec<Rc<BitRef>>, op: Op) -> State {
    assert!(inputs.len() > 0);
    let n = inputs[0].num_bits;
    for a in inputs {
        if *a.read() == State::Unknown {
            return State::Unknown;
        }
    }

    let mut ret = {
        if op == Op::AND {
            vec![true; n]
        } else {
            vec![false; n]
        }
    };

    for a in inputs {
        let a = a.read();
        if let State::Bits(ref a) = *a {
            for i in 0..n {
                match op {
                    Op::AND | Op::NAND => ret[i] = a[i] & ret[i],
                    Op::OR | Op::NOR => ret[i] = a[i] | ret[i],
                    Op::XOR | Op::XNOR => ret[i] = a[i] ^ ret[i],
                    Op::NOT => ret[i] = !a[i],
                }
            }
        } else {
            return State::DontCare;
        }
    }

    for i in 0..n {
        match op {
            Op::NAND => ret[i] = !ret[i],
            Op::NOR => ret[i] = !ret[i],
            Op::XNOR => ret[i] = !ret[i],
            _ => {}
        }
    }
    State::Bits(ret)
}

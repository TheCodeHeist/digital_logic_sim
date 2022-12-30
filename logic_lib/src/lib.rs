enum Direction {
    East,
    West,
    South,
    North,
}

enum GateSize {
    Narrow,
    Medium,
    Wide,
}

enum MultiInputBehavior {
    OneInputOn,
    OneOddNumOn,
}

struct BitRef<const N: usize> {}

struct NotGate<const N: usize> {
    facing: Direction,
    gate_size: GateSize,
    label: String,
    input: BitRef<N>,
    output: bool,
}

struct AndGate {
    facing: Direction,
    gate_size: GateSize,
    label: String,
    inputs: Vec<bool>,
    output: bool,
}

struct OrGate {
    facing: Direction,
    gate_size: GateSize,
    label: String,
    inputs: Vec<bool>,
    output: bool,
}

struct NandGate {
    facing: Direction,
    gate_size: GateSize,
    inputs: Vec<bool>,
    output: bool,
}

struct NorGate {
    facing: Direction,
    gate_size: GateSize,
    label: String,
    inputs: Vec<bool>,
    output: bool,
}

struct XorGate {
    facing: Direction,
    gate_size: GateSize,
    label: String,
    inputs: Vec<bool>,
    output: bool,
    multi_input_behavior: MultiInputBehavior,
}

struct XnorGate {
    facing: Direction,
    gate_size: GateSize,
    label: String,
    inputs: Vec<bool>,
    output: bool,
}

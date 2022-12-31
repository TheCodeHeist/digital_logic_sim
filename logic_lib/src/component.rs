use crate::circuit::Circuit;
use crate::gate::LogicGate;

pub enum Component {
    Gate(LogicGate),
    Circuit(Circuit),
}

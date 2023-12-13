#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct Coordinate {
    pub x: u32,
    pub y: u32,
}

impl std::fmt::Debug for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Coordinate ({}, {})", self.x, self.y)
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct Component {
    pub lib: u32,
    pub name: String,
    pub loc: Coordinate,
    pub id: String,
    pub attributes: std::collections::HashMap<String, String>,
    pub component_type: ComponentType,
}

impl std::fmt::Debug for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "\nComponent {{\n  lib: {},\n  name: {},\n  loc: {:?},\n  id: {},\n  attributes: {:?}\n}}",
            self.lib, self.name, self.loc, self.id, self.attributes
        )
    }
}

pub struct Wire {
    pub from: Coordinate,
    pub to: Coordinate,
}

impl std::fmt::Debug for Wire {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "\nWire {{\n  {:?} -> {:?}\n}}", self.from, self.to)
    }
}

pub enum ModuleType {
    Component(Component),
    WireDestinations(Vec<Coordinate>),
}

#[derive(Eq, PartialEq, Clone)]
pub enum ComponentType {
    AND,
    OR,
    NAND,
    NOR,
    XOR,
    XNOR,
    NOT,
    PIN,
}

impl std::fmt::Debug for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ComponentType::AND => write!(f, "AND"),
            ComponentType::OR => write!(f, "OR"),
            ComponentType::NAND => write!(f, "NAND"),
            ComponentType::NOR => write!(f, "NOR"),
            ComponentType::XOR => write!(f, "XOR"),
            ComponentType::XNOR => write!(f, "XNOR"),
            ComponentType::NOT => write!(f, "NOT"),
            ComponentType::PIN => write!(f, "PIN"),
        }
    }
}

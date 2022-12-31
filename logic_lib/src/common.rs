#[derive(Eq, PartialEq)]
pub enum Direction {
    East,
    West,
    South,
    North,
}

#[derive(Eq, PartialEq)]
pub enum GateSize {
    Narrow,
    Medium,
    Wide,
}

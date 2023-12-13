use std::{collections::HashMap, path::Path};

use xmltree::Element;

use crate::types::{Component, ComponentType, Coordinate, ModuleType, Wire};

pub struct CircParser<'a> {
    file_path: &'a Path,
    components: Vec<Component>,
    wires: Vec<Wire>,
    components_map: HashMap<Coordinate, String>,
    wires_map: HashMap<Coordinate, Vec<Coordinate>>,
    visited_nodes: Vec<Coordinate>,
}

impl CircParser<'_> {
    pub fn new(file_path: &Path) -> CircParser {
        CircParser {
            file_path,
            components: Vec::new(),
            wires: Vec::new(),
            components_map: HashMap::new(),
            wires_map: HashMap::new(),
            visited_nodes: Vec::new(),
        }
    }

    pub fn parse(&mut self) {
        let file_data = std::fs::read_to_string(self.file_path).expect("Failed to read file");
        let parsed =
            Element::parse(file_data.as_bytes()).unwrap_or_else(|_| panic!("Failed to parse"));
        let circuit = parsed
            .get_child("circuit")
            .unwrap_or_else(|| panic!("Failed to get circuit"));

        let mut components: Vec<Component> = Vec::new();
        let mut wires: Vec<Wire> = Vec::new();

        let mut count = 0;
        for child in &circuit.children {
            let elem = child
                .as_element()
                .unwrap_or_else(|| panic!("Failed to get element"));

            match elem.name.as_str() {
                "comp" => {
                    let lib = elem.attributes.get("lib").unwrap();
                    let name = elem.attributes.get("name").unwrap().to_string();
                    let loc = elem.attributes.get("loc").unwrap();
                    let mut attributes: HashMap<String, String> = HashMap::new();
                    let component_type: ComponentType = self.get_component_type(&name);

                    for child in &elem.children {
                        let elem = child
                            .as_element()
                            .unwrap_or_else(|| panic!("Failed to get element"));

                        match elem.name.as_str() {
                            "a" => {
                                let name = elem.attributes.get("name").unwrap();
                                let value = elem.attributes.get("val").unwrap();

                                attributes.insert(String::from(name), String::from(value));
                            }
                            _ => (),
                        }
                    }

                    let id = format!("comp_{}", count);

                    let component = Component {
                        lib: u32::from_str_radix(lib, 10).unwrap(),
                        name,
                        loc: self.parse_string_to_coordinate(loc),
                        id,
                        attributes,
                        component_type,
                    };

                    components.push(component);

                    count += 1;
                }
                "wire" => {
                    let from = elem.attributes.get("from").unwrap();
                    let to = elem.attributes.get("to").unwrap();

                    let wire = Wire {
                        from: self.parse_string_to_coordinate(from),
                        to: self.parse_string_to_coordinate(to),
                    };

                    wires.push(wire);
                }
                _ => (),
            }
        }

        self.components = components;
        self.wires = wires;

        self.map_components_by_location();
        self.map_wires_by_location();
    }

    pub fn get_component_type(&mut self, name: &str) -> ComponentType {
        match name {
            "AND Gate" => ComponentType::AND,
            "OR Gate" => ComponentType::OR,
            "NAND Gate" => ComponentType::NAND,
            "NOR Gate" => ComponentType::NOR,
            "XOR Gate" => ComponentType::XOR,
            "XNOR Gate" => ComponentType::XNOR,
            "NOT Gate" => ComponentType::NOT,
            "Pin" => ComponentType::PIN,
            &_ => todo!("Component type not implemented!"),
        }
    }

    pub fn transpile_to_logic_code(&mut self) -> String {
        let mut logic_code = String::new();

        // DEFINE COMPONENTS
        for component in self.components.clone() {
            // Syntax:
            // #define <id>
            // - type: <type>
            // - <attribute>: <value>
            // - <attribute>: <value>
            // ...

            // #attach <id> <id>
            // #attach <id> <id>
            // ...

            logic_code.push_str(format!("#define {}\n", component.id).as_str());
            logic_code.push_str(format!("- type: {:?}\n", component.component_type).as_str());

            for (attribute, value) in component.attributes {
                logic_code.push_str(format!("- {}: {}\n", attribute, value).as_str());
            }

            logic_code.push_str("\n");
        }

        // CONNECT COMPONENTS
        for component in self.components.clone() {
            let destinations =
                self.track_wire_destinations(ModuleType::Component(component.clone()));

            for destination in destinations {
                logic_code.push_str(
                    format!("#attach {} {}\n", component.id.clone(), destination.clone()).as_str(),
                );
            }
        }

        logic_code
    }

    fn track_wire_destinations(&mut self, module: ModuleType) -> Vec<String> {
        let mut possible_destinations: Vec<String> = Vec::new();

        match module {
            ModuleType::Component(component) => match self.wires_map.get(&component.loc) {
                Some(_) => {
                    self.visited_nodes.push(component.loc);

                    possible_destinations =
                        self.track_wire_destinations(ModuleType::WireDestinations(
                            self.wires_map.get(&component.loc).unwrap().clone(),
                        ));
                }
                None => {
                    panic!("The component does not have any wires connected to it!");
                }
            },

            ModuleType::WireDestinations(routes) => {
                for route in routes {
                    if self.visited_nodes.contains(&route) {
                        continue;
                    }

                    match self.components_map.get(&route) {
                        Some(component_id) => {
                            possible_destinations.push(component_id.clone());
                        }
                        None => match self.wires_map.get(&route) {
                            Some(_) => {
                                self.visited_nodes.push(route);

                                possible_destinations.append(&mut self.track_wire_destinations(
                                    ModuleType::WireDestinations(
                                        self.wires_map.get(&route).unwrap().clone(),
                                    ),
                                ));
                            }
                            None => panic!("The route does not lead to anything!"),
                        },
                    }
                }
            }
        }

        possible_destinations
    }

    fn parse_string_to_coordinate(&mut self, string: &str) -> Coordinate {
        let mut coord = String::from(string).replace("(", "");
        coord = coord.replace(")", "");
        coord = coord.replace(" ", "");
        let mut split = coord.split(",");

        let x = split.next().unwrap().parse::<u32>().unwrap();
        let y = split.next().unwrap().parse::<u32>().unwrap();

        Coordinate { x, y }
    }

    fn calculate_input_coords(&mut self, component: &Component) -> Vec<Coordinate> {
        let mut input_coords: Vec<Coordinate> = Vec::new();

        match component.name.as_str() {
            "AND Gate" | "OR Gate" => {
                // the distance constant is 30
                match component.attributes.get("facing") {
                    Some(facing) => match facing.as_str() {
                        "north" => match component.attributes.get("inputs") {
                            Some(inputs) => {
                                let inputs = inputs.parse::<u32>().unwrap();

                                if inputs % 2 == 0 {
                                    for i in 0..inputs / 2 {
                                        input_coords.push(Coordinate {
                                            x: component.loc.x - (10 * (i + 1)),
                                            y: component.loc.y + 30,
                                        });

                                        input_coords.push(Coordinate {
                                            x: component.loc.x + (10 * (i + 1)),
                                            y: component.loc.y + 30,
                                        });
                                    }
                                } else {
                                    input_coords.push(Coordinate {
                                        x: component.loc.x,
                                        y: component.loc.y + 30,
                                    });

                                    for i in 0..(inputs - 1) / 2 {
                                        input_coords.push(Coordinate {
                                            x: component.loc.x - (10 * (i + 1)),
                                            y: component.loc.y + 30,
                                        });

                                        input_coords.push(Coordinate {
                                            x: component.loc.x + (10 * (i + 1)),
                                            y: component.loc.y + 30,
                                        });
                                    }
                                }
                            }
                            None => {
                                input_coords.push(Coordinate {
                                    x: component.loc.x - 10,
                                    y: component.loc.y + 30,
                                });

                                input_coords.push(Coordinate {
                                    x: component.loc.x + 10,
                                    y: component.loc.y + 30,
                                });
                            }
                        },

                        "south" => match component.attributes.get("inputs") {
                            Some(inputs) => {
                                let inputs = inputs.parse::<u32>().unwrap();

                                if inputs % 2 == 0 {
                                    for i in 0..inputs / 2 {
                                        input_coords.push(Coordinate {
                                            x: component.loc.x - (10 * (i + 1)),
                                            y: component.loc.y - 30,
                                        });

                                        input_coords.push(Coordinate {
                                            x: component.loc.x + (10 * (i + 1)),
                                            y: component.loc.y - 30,
                                        });
                                    }
                                } else {
                                    input_coords.push(Coordinate {
                                        x: component.loc.x,
                                        y: component.loc.y - 30,
                                    });

                                    for i in 0..(inputs - 1) / 2 {
                                        input_coords.push(Coordinate {
                                            x: component.loc.x - (10 * (i + 1)),
                                            y: component.loc.y - 30,
                                        });

                                        input_coords.push(Coordinate {
                                            x: component.loc.x + (10 * (i + 1)),
                                            y: component.loc.y - 30,
                                        });
                                    }
                                }
                            }
                            None => {
                                input_coords.push(Coordinate {
                                    x: component.loc.x - 10,
                                    y: component.loc.y - 30,
                                });

                                input_coords.push(Coordinate {
                                    x: component.loc.x + 10,
                                    y: component.loc.y - 30,
                                });
                            }
                        },

                        "west" => match component.attributes.get("inputs") {
                            Some(inputs) => {
                                let inputs = inputs.parse::<u32>().unwrap();

                                if inputs % 2 == 0 {
                                    for i in 0..inputs / 2 {
                                        input_coords.push(Coordinate {
                                            x: component.loc.x + 30,
                                            y: component.loc.y - (10 * (i + 1)),
                                        });

                                        input_coords.push(Coordinate {
                                            x: component.loc.x + 30,
                                            y: component.loc.y + (10 * (i + 1)),
                                        });
                                    }
                                } else {
                                    input_coords.push(Coordinate {
                                        x: component.loc.x + 30,
                                        y: component.loc.y,
                                    });

                                    for i in 0..(inputs - 1) / 2 {
                                        input_coords.push(Coordinate {
                                            x: component.loc.x + 30,
                                            y: component.loc.y - (10 * (i + 1)),
                                        });

                                        input_coords.push(Coordinate {
                                            x: component.loc.x + 30,
                                            y: component.loc.y + (10 * (i + 1)),
                                        });
                                    }
                                }
                            }
                            None => {
                                input_coords.push(Coordinate {
                                    x: component.loc.x + 30,
                                    y: component.loc.y - 10,
                                });

                                input_coords.push(Coordinate {
                                    x: component.loc.x + 30,
                                    y: component.loc.y + 10,
                                });
                            }
                        },

                        &_ => todo!("Facing not implemented"),
                    },
                    None => match component.attributes.get("inputs") {
                        // Default to East facing
                        Some(inputs) => {
                            let inputs = inputs.parse::<u32>().unwrap();

                            if inputs % 2 == 0 {
                                for i in 0..inputs / 2 {
                                    input_coords.push(Coordinate {
                                        x: component.loc.x - 30,
                                        y: component.loc.y - (10 * (i + 1)),
                                    });

                                    input_coords.push(Coordinate {
                                        x: component.loc.x - 30,
                                        y: component.loc.y + (10 * (i + 1)),
                                    });
                                }
                            } else {
                                input_coords.push(Coordinate {
                                    x: component.loc.x - 30,
                                    y: component.loc.y,
                                });

                                for i in 0..(inputs - 1) / 2 {
                                    input_coords.push(Coordinate {
                                        x: component.loc.x - 30,
                                        y: component.loc.y - (10 * (i + 1)),
                                    });

                                    input_coords.push(Coordinate {
                                        x: component.loc.x - 30,
                                        y: component.loc.y + (10 * (i + 1)),
                                    });
                                }
                            }
                        }

                        None => {
                            input_coords.push(Coordinate {
                                x: component.loc.x - 30,
                                y: component.loc.y - 10,
                            });

                            input_coords.push(Coordinate {
                                x: component.loc.x - 30,
                                y: component.loc.y + 10,
                            });
                        }
                    },
                }
            }

            "NAND Gate" | "NOR Gate" | "XOR Gate" => {
                // the distance constant is 40
                match component.attributes.get("facing") {
                    Some(facing) => match facing.as_str() {
                        "north" => match component.attributes.get("inputs") {
                            Some(inputs) => {
                                let inputs = inputs.parse::<u32>().unwrap();

                                if inputs % 2 == 0 {
                                    for i in 0..inputs / 2 {
                                        input_coords.push(Coordinate {
                                            x: component.loc.x - (10 * (i + 1)),
                                            y: component.loc.y + 40,
                                        });

                                        input_coords.push(Coordinate {
                                            x: component.loc.x + (10 * (i + 1)),
                                            y: component.loc.y + 40,
                                        });
                                    }
                                } else {
                                    input_coords.push(Coordinate {
                                        x: component.loc.x,
                                        y: component.loc.y + 40,
                                    });

                                    for i in 0..(inputs - 1) / 2 {
                                        input_coords.push(Coordinate {
                                            x: component.loc.x - (10 * (i + 1)),
                                            y: component.loc.y + 40,
                                        });

                                        input_coords.push(Coordinate {
                                            x: component.loc.x + (10 * (i + 1)),
                                            y: component.loc.y + 40,
                                        });
                                    }
                                }
                            }
                            None => {
                                input_coords.push(Coordinate {
                                    x: component.loc.x - 10,
                                    y: component.loc.y + 40,
                                });

                                input_coords.push(Coordinate {
                                    x: component.loc.x + 10,
                                    y: component.loc.y + 40,
                                });
                            }
                        },

                        "south" => match component.attributes.get("inputs") {
                            Some(inputs) => {
                                let inputs = inputs.parse::<u32>().unwrap();

                                if inputs % 2 == 0 {
                                    for i in 0..inputs / 2 {
                                        input_coords.push(Coordinate {
                                            x: component.loc.x - (10 * (i + 1)),
                                            y: component.loc.y - 40,
                                        });

                                        input_coords.push(Coordinate {
                                            x: component.loc.x + (10 * (i + 1)),
                                            y: component.loc.y - 40,
                                        });
                                    }
                                } else {
                                    input_coords.push(Coordinate {
                                        x: component.loc.x,
                                        y: component.loc.y - 40,
                                    });

                                    for i in 0..(inputs - 1) / 2 {
                                        input_coords.push(Coordinate {
                                            x: component.loc.x - (10 * (i + 1)),
                                            y: component.loc.y - 40,
                                        });

                                        input_coords.push(Coordinate {
                                            x: component.loc.x + (10 * (i + 1)),
                                            y: component.loc.y - 40,
                                        });
                                    }
                                }
                            }
                            None => {
                                input_coords.push(Coordinate {
                                    x: component.loc.x - 10,
                                    y: component.loc.y - 40,
                                });

                                input_coords.push(Coordinate {
                                    x: component.loc.x + 10,
                                    y: component.loc.y - 40,
                                });
                            }
                        },

                        "west" => match component.attributes.get("inputs") {
                            Some(inputs) => {
                                let inputs = inputs.parse::<u32>().unwrap();

                                if inputs % 2 == 0 {
                                    for i in 0..inputs / 2 {
                                        input_coords.push(Coordinate {
                                            x: component.loc.x + 40,
                                            y: component.loc.y - (10 * (i + 1)),
                                        });

                                        input_coords.push(Coordinate {
                                            x: component.loc.x + 40,
                                            y: component.loc.y + (10 * (i + 1)),
                                        });
                                    }
                                } else {
                                    input_coords.push(Coordinate {
                                        x: component.loc.x + 40,
                                        y: component.loc.y,
                                    });

                                    for i in 0..(inputs - 1) / 2 {
                                        input_coords.push(Coordinate {
                                            x: component.loc.x + 40,
                                            y: component.loc.y - (10 * (i + 1)),
                                        });

                                        input_coords.push(Coordinate {
                                            x: component.loc.x + 40,
                                            y: component.loc.y + (10 * (i + 1)),
                                        });
                                    }
                                }
                            }
                            None => {
                                input_coords.push(Coordinate {
                                    x: component.loc.x + 40,
                                    y: component.loc.y - 10,
                                });

                                input_coords.push(Coordinate {
                                    x: component.loc.x + 40,
                                    y: component.loc.y + 10,
                                });
                            }
                        },

                        &_ => todo!("Facing not implemented"),
                    },

                    None => match component.attributes.get("inputs") {
                        // Default to East facing
                        Some(inputs) => {
                            let inputs = inputs.parse::<u32>().unwrap();

                            if inputs % 2 == 0 {
                                for i in 0..inputs / 2 {
                                    input_coords.push(Coordinate {
                                        x: component.loc.x - 40,
                                        y: component.loc.y - (10 * (i + 1)),
                                    });

                                    input_coords.push(Coordinate {
                                        x: component.loc.x - 40,
                                        y: component.loc.y + (10 * (i + 1)),
                                    });
                                }
                            } else {
                                input_coords.push(Coordinate {
                                    x: component.loc.x - 40,
                                    y: component.loc.y,
                                });

                                for i in 0..(inputs - 1) / 2 {
                                    input_coords.push(Coordinate {
                                        x: component.loc.x - 40,
                                        y: component.loc.y - (10 * (i + 1)),
                                    });

                                    input_coords.push(Coordinate {
                                        x: component.loc.x - 40,
                                        y: component.loc.y + (10 * (i + 1)),
                                    });
                                }
                            }
                        }

                        None => {
                            input_coords.push(Coordinate {
                                x: component.loc.x - 40,
                                y: component.loc.y - 10,
                            });

                            input_coords.push(Coordinate {
                                x: component.loc.x - 40,
                                y: component.loc.y + 10,
                            });
                        }
                    },
                }
            }

            "XNOR Gate" => {
                // the distance constant is 50
                match component.attributes.get("facing") {
                    Some(facing) => match facing.as_str() {
                        "north" => match component.attributes.get("inputs") {
                            Some(inputs) => {
                                let inputs = inputs.parse::<u32>().unwrap();

                                if inputs % 2 == 0 {
                                    for i in 0..inputs / 2 {
                                        input_coords.push(Coordinate {
                                            x: component.loc.x - (10 * (i + 1)),
                                            y: component.loc.y + 50,
                                        });

                                        input_coords.push(Coordinate {
                                            x: component.loc.x + (10 * (i + 1)),
                                            y: component.loc.y + 50,
                                        });
                                    }
                                } else {
                                    input_coords.push(Coordinate {
                                        x: component.loc.x,
                                        y: component.loc.y + 50,
                                    });

                                    for i in 0..(inputs - 1) / 2 {
                                        input_coords.push(Coordinate {
                                            x: component.loc.x - (10 * (i + 1)),
                                            y: component.loc.y + 50,
                                        });

                                        input_coords.push(Coordinate {
                                            x: component.loc.x + (10 * (i + 1)),
                                            y: component.loc.y + 50,
                                        });
                                    }
                                }
                            }
                            None => {
                                input_coords.push(Coordinate {
                                    x: component.loc.x - 10,
                                    y: component.loc.y + 50,
                                });

                                input_coords.push(Coordinate {
                                    x: component.loc.x + 10,
                                    y: component.loc.y + 50,
                                });
                            }
                        },

                        "south" => match component.attributes.get("inputs") {
                            Some(inputs) => {
                                let inputs = inputs.parse::<u32>().unwrap();

                                if inputs % 2 == 0 {
                                    for i in 0..inputs / 2 {
                                        input_coords.push(Coordinate {
                                            x: component.loc.x - (10 * (i + 1)),
                                            y: component.loc.y - 50,
                                        });

                                        input_coords.push(Coordinate {
                                            x: component.loc.x + (10 * (i + 1)),
                                            y: component.loc.y - 50,
                                        });
                                    }
                                } else {
                                    input_coords.push(Coordinate {
                                        x: component.loc.x,
                                        y: component.loc.y - 50,
                                    });

                                    for i in 0..(inputs - 1) / 2 {
                                        input_coords.push(Coordinate {
                                            x: component.loc.x - (10 * (i + 1)),
                                            y: component.loc.y - 50,
                                        });

                                        input_coords.push(Coordinate {
                                            x: component.loc.x + (10 * (i + 1)),
                                            y: component.loc.y - 50,
                                        });
                                    }
                                }
                            }
                            None => {
                                input_coords.push(Coordinate {
                                    x: component.loc.x - 10,
                                    y: component.loc.y - 50,
                                });

                                input_coords.push(Coordinate {
                                    x: component.loc.x + 10,
                                    y: component.loc.y - 50,
                                });
                            }
                        },

                        "west" => match component.attributes.get("inputs") {
                            Some(inputs) => {
                                let inputs = inputs.parse::<u32>().unwrap();

                                if inputs % 2 == 0 {
                                    for i in 0..inputs / 2 {
                                        input_coords.push(Coordinate {
                                            x: component.loc.x + 50,
                                            y: component.loc.y - (10 * (i + 1)),
                                        });

                                        input_coords.push(Coordinate {
                                            x: component.loc.x + 50,
                                            y: component.loc.y + (10 * (i + 1)),
                                        });
                                    }
                                } else {
                                    input_coords.push(Coordinate {
                                        x: component.loc.x + 50,
                                        y: component.loc.y,
                                    });

                                    for i in 0..(inputs - 1) / 2 {
                                        input_coords.push(Coordinate {
                                            x: component.loc.x + 50,
                                            y: component.loc.y - (10 * (i + 1)),
                                        });

                                        input_coords.push(Coordinate {
                                            x: component.loc.x + 50,
                                            y: component.loc.y + (10 * (i + 1)),
                                        });
                                    }
                                }
                            }
                            None => {
                                input_coords.push(Coordinate {
                                    x: component.loc.x + 50,
                                    y: component.loc.y - 10,
                                });

                                input_coords.push(Coordinate {
                                    x: component.loc.x + 50,
                                    y: component.loc.y + 10,
                                });
                            }
                        },

                        &_ => todo!("Facing not implemented"),
                    },

                    None => match component.attributes.get("inputs") {
                        // Default to East facing
                        Some(inputs) => {
                            let inputs = inputs.parse::<u32>().unwrap();

                            if inputs % 2 == 0 {
                                for i in 0..inputs / 2 {
                                    input_coords.push(Coordinate {
                                        x: component.loc.x - 50,
                                        y: component.loc.y - (10 * (i + 1)),
                                    });

                                    input_coords.push(Coordinate {
                                        x: component.loc.x - 50,
                                        y: component.loc.y + (10 * (i + 1)),
                                    });
                                }
                            } else {
                                input_coords.push(Coordinate {
                                    x: component.loc.x - 50,
                                    y: component.loc.y,
                                });

                                for i in 0..(inputs - 1) / 2 {
                                    input_coords.push(Coordinate {
                                        x: component.loc.x - 50,
                                        y: component.loc.y - (10 * (i + 1)),
                                    });

                                    input_coords.push(Coordinate {
                                        x: component.loc.x - 50,
                                        y: component.loc.y + (10 * (i + 1)),
                                    });
                                }
                            }
                        }

                        None => {
                            input_coords.push(Coordinate {
                                x: component.loc.x - 50,
                                y: component.loc.y - 10,
                            });

                            input_coords.push(Coordinate {
                                x: component.loc.x - 50,
                                y: component.loc.y + 10,
                            });
                        }
                    },
                }
            }

            "NOT Gate" => {
                // the distance constant is 20
                match component.attributes.get("facing") {
                    Some(facing) => match facing.as_str() {
                        "north" => {
                            input_coords.push(Coordinate {
                                x: component.loc.x,
                                y: component.loc.y + 20,
                            });
                        }

                        "south" => {
                            input_coords.push(Coordinate {
                                x: component.loc.x,
                                y: component.loc.y - 20,
                            });
                        }

                        "west" => {
                            input_coords.push(Coordinate {
                                x: component.loc.x + 20,
                                y: component.loc.y,
                            });
                        }

                        &_ => todo!("Facing not implemented"),
                    },

                    None => {
                        // Default to East facing
                        input_coords.push(Coordinate {
                            x: component.loc.x - 20,
                            y: component.loc.y,
                        });
                    }
                }
            }

            "Pin" => {
                input_coords.push(Coordinate {
                    x: component.loc.x,
                    y: component.loc.y,
                });
            }

            &_ => todo!("Component not implemented"),
        }

        input_coords
    }

    fn map_components_by_location(&mut self) {
        let mut map: HashMap<Coordinate, String> = HashMap::new();

        for component in &self.components.clone() {
            let all_coords = self.calculate_input_coords(&component);

            for coord in all_coords {
                map.insert(coord, component.id.clone());
            }
        }

        self.components_map = map;
    }

    fn map_wires_by_location(&mut self) {
        let mut map: HashMap<Coordinate, Vec<Coordinate>> = HashMap::new();

        for wire in &self.wires {
            if map.contains_key(&wire.from) {
                let mut coords = map.get(&wire.from).unwrap().clone();
                coords.push(wire.to.clone());

                map.insert(wire.from.clone(), coords);
            } else {
                map.insert(wire.from.clone(), vec![wire.to.clone()]);
            }

            if map.contains_key(&wire.to) {
                let mut coords = map.get(&wire.to).unwrap().clone();
                coords.push(wire.from.clone());

                map.insert(wire.to.clone(), coords);
            } else {
                map.insert(wire.to.clone(), vec![wire.from.clone()]);
            }
        }

        self.wires_map = map;
    }
}

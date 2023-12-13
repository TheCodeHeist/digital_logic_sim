use std::path::Path;

use logic_lib::circ_parser::CircParser;

fn main() {
    let mut circ_parser = CircParser::new(Path::new("./tests/test.circ"));
    circ_parser.parse();
    let generated_code = circ_parser.transpile_to_logic_code();

    // Save the generated code to a file
    std::fs::write("./tests/test.logic", generated_code).expect("Unable to write file");
}

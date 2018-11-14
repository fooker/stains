#![feature(impl_trait_in_bindings)]

use gcode::parser::Parser;

#[test]
fn parse_01() {
    use std::fs::File;
    use std::path::Path;
    use std::io::{BufRead, BufReader};

    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/gcode/01.gcode");
    let file = BufReader::new(File::open(file).unwrap());

    let mut parser = Parser::new();
    for line in file.lines() {
        let block = parser.parse(line.unwrap()).unwrap();
    }
}

#![allow(dead_code)]

extern crate som;

use som::compiler::Parser;
use std::env;
use std::fs::File;
use std::io::BufReader;

#[allow(dead_code)]
fn main() {
    let filename = match env::args().nth(1) {
        Some(f) => f,
        None => panic!("Must provide file to parse"),
    };

    let file = match File::open(&filename) {
        Ok(f) => f,
        Err(e) => panic!("Unable to open {}: {:?}", filename, e),
    };

    let reader = BufReader::new(file);
    let mut parser = Parser::new(reader, filename);
    println!("{:#?}", parser.parse_class().unwrap());
}

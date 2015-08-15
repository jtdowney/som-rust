extern crate som;

use som::compiler::Lexer;
use std::env;
use std::fs::File;
use std::io::BufReader;

#[allow(dead_code)]
fn main() {
    let filename = match env::args().nth(1) {
        Some(f) => f,
        None => panic!("Must provide file to tokenize"),
    };

    let file = match File::open(&filename) {
        Ok(f) => f,
        Err(e) => panic!("Unable to open {}: {:?}", filename, e),
    };

    let reader = BufReader::new(file);
    let lexer = Lexer::new(reader);
    for token in lexer {
        println!("{:?}", token);
    }
}

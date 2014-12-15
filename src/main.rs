use std::io::BufferedReader;
use std::io::fs::File;
use std::os;

mod compiler;

#[allow(dead_code)]
fn main() {
    let args = os::args();
    let file_name = match args.get(1) {
        Some(v) => v,
        None => panic!("Must provide file to tokenize"),
    };

    println!("Tokenizing {}", file_name);
    let file = File::open(&Path::new(file_name)).unwrap();
    let reader = BufferedReader::new(file);
    for token in compiler::lexer::tokenize(reader) {
        println!("Token: {}", token);
    }
}

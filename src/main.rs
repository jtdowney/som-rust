use compiler::lexer::Lexer;
use std::io::{BufferedReader, IoErrorKind};
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
    let mut lexer = Lexer::new(reader);
    loop {
        let token = lexer.read_token();
        match token {
            Ok(t) => println!("{}", t),
            Err(ref e) if e.kind == IoErrorKind::EndOfFile => break,
            Err(e) => panic!("Error during tokenization: {}", e),
        }
    }
}

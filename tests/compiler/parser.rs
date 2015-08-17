use som::compiler::Parser;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

#[test]
fn parse_test_cases() {
    let test_cases_dir = Path::new(file!()).parent().unwrap().join("parser").join("test_cases");
    for entry in fs::read_dir(test_cases_dir).unwrap() {
        let entry = entry.unwrap();
        if entry.path().extension().unwrap() == "som" {
            let computed_file = entry.path();
            let source = BufReader::new(File::open(&computed_file).unwrap());
            let mut parser = Parser::new(source, &computed_file);
            let class = parser.parse_class().unwrap();
            let computed_ast = format!("{:#?}", class);

            let given_file = computed_file.with_extension("som.ast");
            let mut given_ast = String::new();
            let _ = File::open(&given_file).unwrap().read_to_string(&mut given_ast);

            assert_eq!(computed_ast.trim(), given_ast.trim());
        }
    }
}

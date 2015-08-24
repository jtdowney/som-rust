use std::fmt::{Display, Error, Formatter, Write};

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Symbol {
    content: String,
}

impl Symbol {
    pub fn new(content: String) -> Symbol {
        Symbol {
            content: content
        }
    }
}

impl Display for Symbol {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        try!(formatter.write_char('#'));
        try!(formatter.write_str(self.content.as_ref()));
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Symbol;

    #[test]
    fn symbol_equality() {
        let symbol1 = Symbol::new("test".to_string());
        let symbol2 = Symbol::new("test".to_string());
        let symbol3 = Symbol::new("test2".to_string());

        assert!(symbol1 == symbol2);
        assert!(symbol1 != symbol3);
    }

    #[test]
    fn display_format() {
        let symbol = Symbol::new("test".to_string());
        let display = format!("{}", symbol);

        assert_eq!("#test", display);
    }
}

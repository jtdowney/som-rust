use compiler::PeekableBuffer;
use std::io::IoResult;

#[deriving(Show, PartialEq)]
pub enum Token {
    And,
    Assign,
    At,
    Colon,
    Comma,
    Divide,
    Double(f64),
    EndBlock,
    EndTerm,
    Equal,
    Exit,
    Identifier(String),
    Integer(int),
    Keyword(String),
    KeywordSequence(String),
    Less,
    Minus,
    Modulus,
    More,
    NewBlock,
    NewTerm,
    None(char),
    Not,
    OperatorSequence(String),
    Or,
    Percent,
    Period,
    Plus,
    Pound,
    Primitive,
    Separator,
    Star,
    String(String),
}

fn is_operator(c: char) -> bool {
    match c {
        '~' | '&' | '|' | '*' | '/' | '\\' | '+' | '=' | '>' | '<' | ',' | '@' | '%' => true,
        _ => false,
    }
}

pub struct Lexer<B: Buffer> {
    buffer: PeekableBuffer<B>,
}

impl<B: Buffer> Lexer<B> {
    pub fn new(buffer: B) -> Lexer<B> {
        Lexer { buffer: PeekableBuffer::new(buffer) }
    }

    pub fn read_token(&mut self) -> IoResult<Token> {
        loop {
            try!(self.skip_whitespace());
            try!(self.skip_comment());

            let c = try!(self.buffer.peek_char());
            if c.is_whitespace() || c == '"' {
                continue;
            } else {
                break;
            }
        }

        let c = try!(self.buffer.peek_char());
        match c {
            '[' => self.tokenize_token(Token::NewBlock),
            ']' => self.tokenize_token(Token::EndBlock),
            '(' => self.tokenize_token(Token::NewTerm),
            ')' => self.tokenize_token(Token::EndTerm),
            '#' => self.tokenize_token(Token::Pound),
            '^' => self.tokenize_token(Token::Exit),
            '.' => self.tokenize_token(Token::Period),
            '\'' => self.tokenize_string(),
            '0'...'9' => self.tokenize_number(),
            ':' => self.tokenize_colon(),
            '-' => self.tokenize_minus(),
            'a'...'z' | 'A'...'Z' => self.tokenize_identifier(),
            _ => {
                if is_operator(c) {
                    self.tokenize_operator()
                } else {
                    self.tokenize_token(Token::None(c))
                }
            }
        }
    }

    fn skip_whitespace(&mut self) -> IoResult<()> {
        loop {
            let c = try!(self.buffer.peek_char());
            if c.is_whitespace() {
                try!(self.buffer.consume());
            } else {
                break;
            }
        }
        Ok(())
    }

    fn skip_comment(&mut self) -> IoResult<()> {
        if self.buffer.peek_char() != Ok('"') {
            return Ok(());
        }

        try!(self.buffer.consume());
        loop {
            if self.buffer.read_char() == Ok('"') {
                break;
            }
        }

        Ok(())
    }

    fn tokenize_token(&mut self, token: Token) -> IoResult<Token> {
        try!(self.buffer.consume());
        Ok(token)
    }

    fn tokenize_colon(&mut self) -> IoResult<Token> {
        try!(self.buffer.consume());
        if self.buffer.peek_char() == Ok('=') {
            self.tokenize_token(Token::Assign)
        } else {
            Ok(Token::Colon)
        }
    }

    fn tokenize_minus(&mut self) -> IoResult<Token> {
        try!(self.buffer.consume());
        if self.buffer.peek_char() == Ok('-') && self.buffer.peek_peek_char() == Ok('-') {
            loop {
                if self.buffer.read_char() != Ok('-') {
                    break;
                }
            }
            Ok(Token::Separator)
        } else {
            Ok(Token::Minus)
        }
    }

    fn tokenize_identifier(&mut self) -> IoResult<Token> {
        let mut text = String::new();

        loop {
            if self.buffer.is_eof() {
                break;
            }

            let c = try!(self.buffer.peek_char());
            match c {
                'a'...'z' | 'A'...'Z' | '0'...'9' | '_' => {
                    text.push(c);
                    try!(self.buffer.consume());
                }
                _ => break,
            }
        }

        if self.buffer.peek_char() == Ok(':') {
            try!(self.buffer.consume());
            text.push(':');

            let saw_sequence = self.buffer.peek_char().ok().and_then(|c| {
                Some(c.is_alphabetic())
            }).unwrap_or(false);
            if saw_sequence {
                loop {
                    if self.buffer.is_eof() {
                        break;
                    }

                    let c = try!(self.buffer.peek_char());
                    match c {
                        'a'...'z' | 'A'...'Z' | ':' => {
                            text.push(c);
                            try!(self.buffer.consume());
                        }
                        _ => break,
                    }
                }

                Ok(Token::KeywordSequence(text))
            } else {
                Ok(Token::Keyword(text))
            }
        } else if text == "primitive" {
            Ok(Token::Primitive)
        } else {
            Ok(Token::Identifier(text))
        }
    }

    fn tokenize_number(&mut self) -> IoResult<Token> {
        let mut text = String::new();

        loop {
            if self.buffer.is_eof() {
                break;
            }

            let c = try!(self.buffer.peek_char());
            match c {
                '0'...'9' => {
                    text.push(c);
                    try!(self.buffer.consume());
                }
                _ => break,
            }
        }

        let saw_decimal = self.buffer.peek_char().ok().and_then(|c| {
            if c == '.' {
                self.buffer.peek_peek_char().ok()
            } else {
                None
            }
        }).and_then(|c| {
            Some(c.is_digit(10))
        }).unwrap_or(false);

        if saw_decimal {
            try!(self.buffer.consume());
            text.push('.');
            loop {
                if self.buffer.is_eof() {
                    break;
                }

                let c = try!(self.buffer.peek_char());
                match c {
                    '0'...'9' => {
                        text.push(c);
                        try!(self.buffer.consume());
                    }
                    _ => break,
                }
            }

            let value = from_str(text.as_slice()).unwrap();
            Ok(Token::Double(value))
        } else {
            let value = from_str(text.as_slice()).unwrap();
            Ok(Token::Integer(value))
        }
    }

    fn tokenize_string(&mut self) -> IoResult<Token> {
        let mut text = String::new();

        try!(self.buffer.consume());
        loop {
            let c = try!(self.buffer.read_char());
            if c == '\'' {
                break;
            } else {
                text.push(c);
            }
        }

        Ok(Token::String(text))
    }

    fn tokenize_operator(&mut self) -> IoResult<Token> {
        let c = try!(self.buffer.read_char());
        let saw_operator = self.buffer.peek_char().ok().and_then(|c| {
            Some(is_operator(c))
        }).unwrap_or(false);

        if saw_operator {
            let mut text = String::from_chars(&[c]);
            loop {
                if self.buffer.is_eof() {
                    break;
                }

                let c = try!(self.buffer.peek_char());
                if is_operator(c) {
                    text.push(c);
                    try!(self.buffer.consume());
                } else {
                    break;
                }
            }

            Ok(Token::OperatorSequence(text))
        } else {
            match c {
                '~' => Ok(Token::Not),
                '&' => Ok(Token::And),
                '|' => Ok(Token::Or),
                '*' => Ok(Token::Star),
                '/' => Ok(Token::Divide),
                '\\' => Ok(Token::Modulus),
                '+' => Ok(Token::Plus),
                '=' => Ok(Token::Equal),
                '>' => Ok(Token::More),
                '<' => Ok(Token::Less),
                ',' => Ok(Token::Comma),
                '@' => Ok(Token::At),
                '%' => Ok(Token::Percent),
                _ => panic!("Could not identify operator: {}", c)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Lexer, Token, tokenize};

    #[test]
    fn tokenize_iterates() {
        let source = "abc".as_bytes();
        let mut iterator = tokenize(source);
        assert_eq!(iterator.next(), Some(Token::Identifier("abc".to_string())));
    }

    #[test]
    fn tokenization_skips_whitespace() {
        let source = "\n Hello".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Identifier("Hello".to_string())));
    }

    #[test]
    fn tokenization_skips_comments() {
        let source = "\"comment\" Hello".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Identifier("Hello".to_string())));
    }

    #[test]
    fn tokenizes_identifier() {
        let source = "Hello".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Identifier("Hello".to_string())));
    }

    #[test]
    fn tokenizes_keyword() {
        let source = "foo:".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Keyword("foo:".to_string())));
    }

    #[test]
    fn tokenizes_two_keyword_sequence() {
        let source = "foo:bar:".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::KeywordSequence("foo:bar:".to_string())));
    }

    #[test]
    fn tokenizes_three_keyword_sequence() {
        let source = "foo:bar:baz:".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::KeywordSequence("foo:bar:baz:".to_string())));
    }

    #[test]
    fn tokenizes_primitive() {
        let source = "primitive".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Primitive));
    }

    #[test]
    fn tokenizes_colon() {
        let source = ":".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Colon));
    }

    #[test]
    fn tokenizes_pound() {
        let source = "#".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Pound));
    }

    #[test]
    fn tokenizes_exit() {
        let source = "^".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Exit));
    }

    #[test]
    fn tokenizes_period() {
        let source = ".".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Period));
    }

    #[test]
    fn tokenizes_minus() {
        let source = "-".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Minus));
    }

    #[test]
    fn tokenizes_separator() {
        let source = "----".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Separator));
    }

    #[test]
    fn tokenizes_long_separator() {
        let source = "--------".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Separator));
    }

    #[test]
    fn tokenizes_new_term() {
        let source = "(".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::NewTerm));
    }

    #[test]
    fn tokenizes_end_term() {
        let source = ")".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::EndTerm));
    }

    #[test]
    fn tokenizes_new_block() {
        let source = "[".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::NewBlock));
    }

    #[test]
    fn tokenizes_end_block() {
        let source = "]".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::EndBlock));
    }

    #[test]
    fn tokenizes_none() {
        let source = "\u{00FE}".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::None('\u{00FE}')));
    }

    #[test]
    fn tokenizes_string() {
        let source = "'Hello'".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::String("Hello".to_string())));
    }

    #[test]
    fn tokenizes_integer() {
        let source = "1".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Integer(1)));
    }

    #[test]
    fn tokenizes_integer_and_period() {
        let source = "1.".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Integer(1)));
        assert_eq!(lexer.read_token(), Ok(Token::Period));
    }

    #[test]
    fn tokenizes_double() {
        let source = "3.14".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Double(3.14)));
    }

    #[test]
    fn tokenizes_assignment() {
        let source = "foo := 'Hello'".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Identifier("foo".to_string())));
        assert_eq!(lexer.read_token(), Ok(Token::Assign));
        assert_eq!(lexer.read_token(), Ok(Token::String("Hello".to_string())));
    }

    #[test]
    fn tokenizes_operator_sequence() {
        let source = ">=".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::OperatorSequence(">=".to_string())));
    }

    #[test]
    fn tokenizes_not() {
        let source = "~".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Not));
    }

    #[test]
    fn tokenizes_and() {
        let source = "&".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::And));
    }

    #[test]
    fn tokenizes_or() {
        let source = "|".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Or));
    }

    #[test]
    fn tokenizes_star() {
        let source = "*".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Star));
    }

    #[test]
    fn tokenizes_div() {
        let source = "/".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Divide));
    }

    #[test]
    fn tokenizes_mod() {
        let source = "\\".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Modulus));
    }

    #[test]
    fn tokenizes_plus() {
        let source = "+".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Plus));
    }

    #[test]
    fn tokenizes_equal() {
        let source = "=".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Equal));
    }

    #[test]
    fn tokenizes_more() {
        let source = ">".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::More));
    }

    #[test]
    fn tokenizes_less() {
        let source = "<".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Less));
    }

    #[test]
    fn tokenizes_comma() {
        let source = ",".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Comma));
    }

    #[test]
    fn tokenizes_at() {
        let source = "@".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::At));
    }

    #[test]
    fn tokenizes_percent() {
        let source = "%".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Percent));
    }

    #[test]
    fn tokenizes_simple_file() {
        let source = "
        Hello = (
            \"The 'run' method is called when initializing the system\"
            run = ('Hello, World from SOM' println )
        )
        ".as_bytes();
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.read_token(), Ok(Token::Identifier("Hello".to_string())));
        assert_eq!(lexer.read_token(), Ok(Token::Equal));
        assert_eq!(lexer.read_token(), Ok(Token::NewTerm));
        assert_eq!(lexer.read_token(), Ok(Token::Identifier("run".to_string())));
        assert_eq!(lexer.read_token(), Ok(Token::Equal));
        assert_eq!(lexer.read_token(), Ok(Token::NewTerm));
        assert_eq!(lexer.read_token(), Ok(Token::String("Hello, World from SOM".to_string())));
        assert_eq!(lexer.read_token(), Ok(Token::Identifier("println".to_string())));
        assert_eq!(lexer.read_token(), Ok(Token::EndTerm));
        assert_eq!(lexer.read_token(), Ok(Token::EndTerm));
    }
}

use compiler::{ast, Lexer, Symbol, Token};
use compiler::lexer::Item;
use util::peekable_buffer::Location;
use std::collections::{HashMap, VecDeque};
use std::io::BufRead;
use std::iter::Peekable;
use std::path::Path;

const BINARY_OPERATORS: [Symbol; 14] = [
    Symbol::And, Symbol::At, Symbol::Comma, Symbol::Divide, Symbol::Equal,
    Symbol::Less, Symbol::Minus, Symbol::Modulus, Symbol::More, Symbol::Not,
    Symbol::Or, Symbol::Percent, Symbol::Plus, Symbol::Star,
];

fn is_binary_operator(symbol: &Symbol) -> bool {
    BINARY_OPERATORS.contains(symbol)
}

fn binary_symbol_to_string(symbol: &Symbol) -> String {
    match symbol {
        &Symbol::And     => "&",
        &Symbol::At      => "@",
        &Symbol::Comma   => ",",
        &Symbol::Divide  => "/",
        &Symbol::Equal   => "=",
        &Symbol::Less    => "<",
        &Symbol::Minus   => "-",
        &Symbol::Modulus => "\\",
        &Symbol::More    => ">",
        &Symbol::Not     => "~",
        &Symbol::Or      => "|",
        &Symbol::Percent => "%",
        &Symbol::Plus    => "+",
        &Symbol::Star    => "*",
        _                => unreachable!(),
    }.to_string()
}

#[derive(Debug, PartialEq)]
pub enum Error {
    ParseError { description: String, filename: String, line: usize, position: usize },
    MismatchError { expected: Vec<Symbol>, found: Symbol, location: Location },
    End
}

pub struct Parser<R: BufRead, P: AsRef<Path>> {
    lexer: Peekable<Lexer<R>>,
    queue: VecDeque<Item>,
    filename: P,
}

impl<R: BufRead, P: AsRef<Path>> Parser<R, P> {
    pub fn new(reader: R, filename: P) -> Parser<R, P> {
        Parser {
            lexer: Lexer::new(reader).peekable(),
            queue: VecDeque::new(),
            filename: filename,
        }
    }

    pub fn parse_class(&mut self) -> Result<ast::Class, Error> {
        let name = try!(self.expect(Symbol::Identifier)).unwrap();
        try!(self.expect(Symbol::Equal));
        let superclass = try!(self.parse_superclass_name());
        try!(self.expect(Symbol::NewTerm));

        let instance_variables = try!(self.parse_locals());
        let mut instance_methods = HashMap::new();
        loop {
            let (name, method) = match try!(self.peek(1)) {
                Token(Symbol::Identifier, _) => try!(self.parse_method()),
                Token(Symbol::Keyword, _) => try!(self.parse_method()),
                Token(Symbol::OperatorSequence, _) => try!(self.parse_method()),
                Token(ref symbol, _) if is_binary_operator(symbol) => try!(self.parse_method()),
                _ => break,
            };

            instance_methods.insert(name, method);
        }

        let mut class_methods = HashMap::new();
        let mut class_variables = vec![];
        if self.accept(Symbol::Separator).is_ok() {
            class_variables = try!(self.parse_locals());
            loop {
                let (name, method) = match try!(self.peek(1)) {
                    Token(Symbol::Identifier, _) => try!(self.parse_method()),
                    Token(Symbol::Keyword, _) => try!(self.parse_method()),
                    Token(Symbol::OperatorSequence, _) => try!(self.parse_method()),
                    Token(ref symbol, _) if is_binary_operator(symbol) => try!(self.parse_method()),
                    _ => break,
                };

                class_methods.insert(name, method);
            }
        }

        try!(self.expect(Symbol::EndTerm));

        Ok(ast::Class {
            name: name,
            superclass: superclass,
            instance_methods: instance_methods,
            instance_variables: instance_variables,
            class_methods: class_methods,
            class_variables: class_variables,
        })
    }

    fn parse_superclass_name(&mut self) -> Result<String, Error> {
        match self.accept(Symbol::Identifier) {
            Ok(Token(Symbol::Identifier, text)) => Ok(text.unwrap()),
            Ok(_) => unreachable!(),
            Err(Error::MismatchError { expected: _, found: _, location: _ }) => Ok("Object".to_string()),
            Err(e) => Err(e),
        }
    }

    fn parse_method(&mut self) -> Result<(String, ast::Method), Error> {
        let (name, parameters) = try!(self.parse_pattern());
        try!(self.expect(Symbol::Equal));

        if self.accept(Symbol::Primitive).is_ok() {
            let method = ast::Method::Primitive {
                name: name.clone(),
                parameters: parameters,
            };
            Ok((name, method))
        } else {
            try!(self.expect(Symbol::NewTerm));
            let method = ast::Method::Native {
                name: name.clone(),
                parameters: parameters,
                locals: try!(self.parse_locals()),
                body: try!(self.parse_block_body()),
            };
            try!(self.expect(Symbol::EndTerm));
            Ok((name, method))
        }
    }

    fn parse_pattern(&mut self) -> Result<(String, Vec<String>), Error> {
        match self.peek(1) {
            Ok(Token(Symbol::Identifier, _)) => self.parse_unary_pattern(),
            Ok(Token(Symbol::Keyword, _)) => self.parse_keyword_pattern(),
            Ok(Token(Symbol::OperatorSequence, _)) => self.parse_binary_pattern(),
            Ok(Token(ref symbol, _)) if is_binary_operator(symbol) => self.parse_binary_pattern(),
            _ => unreachable!(),
        }
    }

    fn parse_unary_pattern(&mut self) -> Result<(String, Vec<String>), Error> {
        let name = try!(self.expect(Symbol::Identifier)).unwrap();
        Ok((name, vec![]))
    }

    fn parse_keyword_pattern(&mut self) -> Result<(String, Vec<String>), Error> {
        let mut name = try!(self.expect(Symbol::Keyword)).unwrap();
        let mut parameters = vec![];
        parameters.push(try!(self.expect(Symbol::Identifier)).unwrap());
        loop {
            match self.accept(Symbol::Keyword) {
                Ok(Token(Symbol::Keyword, text)) => {
                    name.push_str(text.unwrap().as_ref());
                    parameters.push(try!(self.expect(Symbol::Identifier)).unwrap());
                },
                Ok(_) => unreachable!(),
                Err(Error::MismatchError { expected: _, found: _, location: _ }) => break,
                Err(e) => return Err(e),
            }
        }

        Ok((name, parameters))
    }

    fn parse_binary_pattern(&mut self) -> Result<(String, Vec<String>), Error> {
        let name = match self.peek(1) {
            Ok(Token(Symbol::OperatorSequence, text)) => text.unwrap(),
            Ok(Token(ref symbol, _)) if is_binary_operator(symbol) => binary_symbol_to_string(symbol),
            _ => unreachable!(),
        };

        try!(self.consume(1));
        let parameter = try!(self.expect(Symbol::Identifier)).unwrap();

        Ok((name, vec![parameter]))
    }

    fn parse_locals(&mut self) -> Result<Vec<String>, Error> {
        let mut locals = Vec::new();
        if self.accept(Symbol::Or).is_ok() {
            loop {
                match self.accept(Symbol::Identifier) {
                    Ok(Token(_, text)) => locals.push(text.unwrap()),
                    _ => break,
                }
            }

            try!(self.expect(Symbol::Or));
        }

        Ok(locals)
    }

    fn parse_block_parameters(&mut self) -> Result<Vec<String>, Error> {
        let mut parameters = vec![];
        loop {
            if self.peek(1) == Ok(Token(Symbol::Colon, None)) {
                try!(self.expect(Symbol::Colon));
                let parameter = try!(self.expect(Symbol::Identifier)).unwrap();
                parameters.push(parameter);
            } else {
                if !parameters.is_empty() {
                    try!(self.expect(Symbol::Or));
                }

                break;
            }
        }

        Ok(parameters)
    }

    fn parse_block_body(&mut self) -> Result<Vec<ast::Expression>, Error> {
        let mut statements = Vec::new();

        loop {
            match self.peek(1) {
                Ok(Token(Symbol::EndTerm, _)) => break,
                Ok(Token(Symbol::EndBlock, _)) => break,
                Ok(Token(Symbol::Exit, _)) => statements.push(try!(self.parse_result())),
                Ok(_) => statements.push(try!(self.parse_expression())),
                Err(Error::End) => break,
                Err(_) => unreachable!(),
            };

            if self.accept(Symbol::Period).is_ok() {
                continue;
            } else {
                break;
            }
        }

        Ok(statements)
    }

    fn parse_result(&mut self) -> Result<ast::Expression, Error> {
        try!(self.expect(Symbol::Exit));
        let statement = Box::new(try!(self.parse_expression()));
        Ok(ast::Expression::Return(statement))
    }

    fn parse_assignments(&mut self) -> Result<Vec<String>, Error> {
        let mut assignments = vec![];

        loop {
            if self.peek(2) == Ok(Token(Symbol::Assign, None)) {
                assignments.push(try!(self.expect(Symbol::Identifier)).unwrap());
                try!(self.expect(Symbol::Assign));
            } else {
                break;
            }
        }

        Ok(assignments)
    }

    fn parse_expression(&mut self) -> Result<ast::Expression, Error> {
        if self.peek(2) == Ok(Token(Symbol::Assign, None)) {
            Ok(ast::Expression::Assignment {
                variables: try!(self.parse_assignments()),
                value: Box::new(try!(self.parse_expression())),
            })
        } else {
            let mut expression = try!(self.parse_expression_primary());

            loop {
                expression = match self.peek(1) {
                    Ok(Token(Symbol::Identifier, _)) => try!(self.parse_expression_messages(expression)),
                    Ok(Token(Symbol::Keyword, _)) => try!(self.parse_expression_messages(expression)),
                    Ok(Token(Symbol::OperatorSequence, _)) => try!(self.parse_expression_messages(expression)),
                    Ok(Token(ref symbol, _)) if is_binary_operator(symbol) => try!(self.parse_expression_messages(expression)),
                    _ => break,
                }
            }

            Ok(expression)
        }
    }

    fn parse_expression_primary(&mut self) -> Result<ast::Expression, Error> {
        match self.peek(1) {
            Ok(Token(Symbol::Identifier, _)) => self.parse_expression_variable(),
            Ok(Token(Symbol::String, _)) => self.parse_expression_string(),
            Ok(Token(Symbol::Integer, _)) => self.parse_expression_number(false),
            Ok(Token(Symbol::Double, _)) => self.parse_expression_number(false),
            Ok(Token(Symbol::Pound, _)) => self.parse_expression_symbol(),
            Ok(Token(Symbol::Minus, _)) => self.parse_expression_negative_number(),
            Ok(Token(Symbol::NewBlock, _)) => self.parse_expression_nested_block(),
            Ok(Token(Symbol::NewTerm, _)) => self.parse_expression_nested_term(),
            Ok(t) => unreachable!(format!("token: {:#?}", t)),
            Err(e) => Err(e),
        }
    }

    fn parse_expression_messages(&mut self, value: ast::Expression) -> Result<ast::Expression, Error> {
        let mut expression = value;
        let Token(symbol, _) = try!(self.peek(1));
        match symbol {
            Symbol::Identifier => {
                loop {
                    if let Ok(Token(Symbol::Identifier, _)) = self.peek(1) {
                        expression = try!(self.parse_expression_unary_message(expression));
                    } else {
                        break;
                    }
                }

                Ok(expression)
            }
            Symbol::Keyword => self.parse_expression_keyword_message(expression),
            Symbol::OperatorSequence => self.parse_expression_binary_message(expression),
            ref s if is_binary_operator(s) => self.parse_expression_binary_message(expression),
            _ => unreachable!(),
        }
    }

    fn parse_expression_nested_block(&mut self) -> Result<ast::Expression, Error> {
        try!(self.expect(Symbol::NewBlock));
        let value = ast::Expression::Block {
            parameters: try!(self.parse_block_parameters()),
            locals: try!(self.parse_locals()),
            body: try!(self.parse_block_body()),
        };
        try!(self.expect(Symbol::EndBlock));

        Ok(value)
    }

    fn parse_expression_nested_term(&mut self) -> Result<ast::Expression, Error> {
        try!(self.expect(Symbol::NewTerm));
        let value = try!(self.parse_expression());
        try!(self.expect(Symbol::EndTerm));

        Ok(value)
    }

    fn parse_expression_variable(&mut self) -> Result<ast::Expression, Error> {
        let variable = try!(self.expect(Symbol::Identifier)).unwrap();
        let value = match variable.as_ref() {
            "nil" => ast::Expression::LiteralNil,
            "true" => ast::Expression::LiteralBoolean(true),
            "false" => ast::Expression::LiteralBoolean(false),
            _ => ast::Expression::Variable(variable),
        };

        Ok(value)
    }

    fn parse_expression_string(&mut self) -> Result<ast::Expression, Error> {
        let value = try!(self.expect(Symbol::String)).unwrap();
        Ok(ast::Expression::LiteralString(value))
    }

    fn parse_expression_symbol(&mut self) -> Result<ast::Expression, Error> {
        try!(self.expect(Symbol::Pound));

        let value = match self.peek(1) {
            Ok(Token(Symbol::Identifier, text)) => text.unwrap(),
            Ok(Token(Symbol::String, text)) => text.unwrap(),
            Ok(Token(Symbol::Keyword, text)) => text.unwrap(),
            Ok(Token(Symbol::KeywordSequence, text)) => text.unwrap(),
            Ok(Token(Symbol::OperatorSequence, text)) => text.unwrap(),
            Ok(Token(ref symbol, _)) if is_binary_operator(symbol) => binary_symbol_to_string(symbol),
            _ => unreachable!(),
        };

        try!(self.consume(1));

        Ok(ast::Expression::LiteralSymbol(value))
    }

    fn parse_expression_negative_number(&mut self) -> Result<ast::Expression, Error> {
        try!(self.expect(Symbol::Minus));
        self.parse_expression_number(true)
    }

    fn parse_expression_number(&mut self, negative: bool) -> Result<ast::Expression, Error> {
        match self.accept_one_of(&[Symbol::Integer, Symbol::Double]) {
            Ok(Token(Symbol::Integer, Some(text))) => {
                let mut value: i64 = text.parse().unwrap();
                if negative {
                    value = -value;
                }

                Ok(ast::Expression::LiteralInteger(value))
            },
            Ok(Token(Symbol::Double, Some(text))) => {
                let mut value: f64 = text.parse().unwrap();
                if negative {
                    value = -value;
                }

                Ok(ast::Expression::LiteralDouble(value))
            },
            Ok(_) => unreachable!(),
            Err(e) => Err(e),
        }
    }

    fn parse_expression_unary_message(&mut self, value: ast::Expression) -> Result<ast::Expression, Error> {
        let message = try!(self.expect(Symbol::Identifier)).unwrap();
        Ok(ast::Expression::UnaryMessage { receiver: Box::new(value), message: message })
    }

    fn parse_expression_keyword_message(&mut self, value: ast::Expression) -> Result<ast::Expression, Error> {
        let mut message = String::new();
        let mut parameters = Vec::new();
        loop {
            match self.accept(Symbol::Keyword) {
                Ok(Token(Symbol::Keyword, text)) => {
                    message.push_str(text.unwrap().as_ref());
                    parameters.push(try!(self.parse_expression_formula()));
                }
                Ok(_) => unreachable!(),
                Err(_) => break,
            };
        }

        Ok(ast::Expression::KeywordMessage {
            receiver: Box::new(value),
            message: message,
            parameters: parameters,
        })
    }

    fn parse_expression_formula(&mut self) -> Result<ast::Expression, Error> {
        let mut value = try!(self.parse_expression_binary_operand());

        loop {
            match self.peek(1) {
                Ok(Token(ref symbol, _)) if is_binary_operator(symbol) => {
                    value = try!(self.parse_expression_binary_message(value));
                }
                Ok(Token(Symbol::OperatorSequence, _)) => {
                    value = try!(self.parse_expression_binary_message(value));
                }
                _ => break,
            }
        }

        Ok(value)
    }

    fn parse_expression_binary_operand(&mut self) -> Result<ast::Expression, Error> {
        let mut value = try!(self.parse_expression_primary());

        loop {
            match self.peek(1) {
                Ok(Token(Symbol::Identifier, _)) => {
                    value = try!(self.parse_expression_unary_message(value));
                }
                _ => break,
            }
        }

        Ok(value)
    }

    fn parse_expression_binary_message(&mut self, value: ast::Expression) -> Result<ast::Expression, Error> {
        let message = match self.peek(1) {
            Ok(Token(ref symbol, _)) if is_binary_operator(symbol) => binary_symbol_to_string(symbol),
            Ok(Token(Symbol::OperatorSequence, text)) => text.unwrap(),
            Ok(_) => unreachable!(),
            Err(e) => return Err(e),
        };

        try!(self.consume(1));

        Ok(ast::Expression::BinaryMessage {
            message: message,
            left: Box::new(value),
            right: Box::new(try!(self.parse_expression_binary_operand())),
        })
    }

    fn peek(&mut self, n: usize) -> Result<Token, Error> {
        for _ in (self.queue.len()..n) {
            match self.lexer.next() {
                Some(t) => self.queue.push_back(t),
                None => return Err(Error::End),
            }
        }

        match self.queue.get(n-1) {
            Some(t) => Ok(t.0.clone()),
            None => Err(Error::End),
        }
    }

    fn consume(&mut self, n: usize) -> Result<(), Error> {
        for _ in (0..n) {
            if self.queue.is_empty() {
                self.lexer.next();
            } else {
                self.queue.pop_front();
            };
        }

        Ok(())
    }

    fn accept(&mut self, expected: Symbol) -> Result<Token, Error> {
        self.accept_one_of(&[expected])
    }

    fn accept_one_of(&mut self, expected: &[Symbol]) -> Result<Token, Error> {
        let result = {
            let next_item = if self.queue.is_empty() {
                self.lexer.peek()
            } else {
                self.queue.front()
            };

            match next_item {
                Some(&Item(Token(ref symbol, ref text), ref location)) => {
                    if expected.contains(&symbol) {
                        Ok(Token(symbol.clone(), text.clone()))
                    } else {
                        Err(Error::MismatchError { expected: expected.to_owned(), found: symbol.clone(), location: *location })
                    }
                }
                None => Err(Error::End),
            }
        };

        if result.is_ok() {
            if self.queue.is_empty() {
                self.lexer.next();
            } else {
                self.queue.pop_front();
            };
        }

        result
    }

    fn expect(&mut self, expected: Symbol) -> Result<Option<String>, Error> {
        self.expect_one_of(&[expected])
    }

    fn expect_one_of(&mut self, expected: &[Symbol]) -> Result<Option<String>, Error> {
        match self.accept_one_of(expected) {
            Ok(Token(_, text)) => Ok(text),
            Err(Error::MismatchError { expected, found, location }) => Err(Error::ParseError {
                description: format!("Expected {:?}, found {:?}", expected, found),
                filename: self.filename.as_ref().to_string_lossy().into_owned(),
                line: location.0,
                position: location.1,
            }),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use compiler::ast;
    use compiler::Symbol;
    use super::{Error, Parser};

    #[test]
    fn parse_error() {
        let source = "Hello".as_bytes();
        let mut parser = Parser::new(source, "test");
        let result = parser.expect(Symbol::Double);
        assert_eq!(result, Err(Error::ParseError {
            description: "Expected [Double], found Identifier".to_string(),
            filename: "test".to_string(),
            line: 1,
            position: 1,
        }));
    }

    #[test]
    fn parse_error_position_information() {
        let source = " \n  World".as_bytes();
        let mut parser = Parser::new(source, "test");
        let result = parser.expect(Symbol::Double);
        assert_eq!(result, Err(Error::ParseError {
            description: "Expected [Double], found Identifier".to_string(),
            filename: "test".to_string(),
            line: 2,
            position: 3,
        }));
    }

    #[test]
    fn parse_method_primitive() {
        let source = "hello = primitive".as_bytes();
        let mut parser = Parser::new(source, "test");
        let (_, method) = parser.parse_method().unwrap();
        assert_eq!(method, ast::Method::Primitive { name: "hello".to_string(), parameters: vec![] });
    }

    #[test]
    fn parse_assignment() {
        let source = "a := 'test'".as_bytes();
        let mut parser = Parser::new(source, "test");
        let statements = parser.parse_block_body().unwrap();
        let statement = statements.first().unwrap();
        assert_eq!(statement, &ast::Expression::Assignment {
            variables: vec!["a".to_string()],
            value: Box::new(ast::Expression::LiteralString("test".to_string())),
        });
    }

    #[test]
    fn parse_multiple_assignment() {
        let source = "a := b := 'test'".as_bytes();
        let mut parser = Parser::new(source, "test");
        let statements = parser.parse_block_body().unwrap();
        let statement = statements.first().unwrap();
        assert_eq!(statement, &ast::Expression::Assignment {
            variables: vec!["a".to_string(), "b".to_string()],
            value: Box::new(ast::Expression::LiteralString("test".to_string())),
        });
    }

    #[test]
    fn parse_body_evaluation() {
        let source = "'test' println".as_bytes();
        let mut parser = Parser::new(source, "test");
        let statements = parser.parse_block_body().unwrap();
        let statement = statements.first().unwrap();
        assert_eq!(statement, &ast::Expression::UnaryMessage {
            message: "println".to_string(),
            receiver: Box::new(ast::Expression::LiteralString("test".to_string())),
        });
    }

    #[test]
    fn nested_block_expression() {
        let source = "[ :arg | arg print. ' ' print ]".as_bytes();
        let mut parser = Parser::new(source, "test");
        let expression = parser.parse_expression().unwrap();
        assert_eq!(expression, ast::Expression::Block {
            parameters: vec!["arg".to_string()],
            locals: vec![],
            body: vec![
                ast::Expression::UnaryMessage {
                    message: "print".to_string(),
                    receiver: Box::new(ast::Expression::Variable("arg".to_string())),
                },
                ast::Expression::UnaryMessage {
                    message: "print".to_string(),
                    receiver: Box::new(ast::Expression::LiteralString(" ".to_string())),
                },
            ],
        });
    }

    #[test]
    fn variable_expression() {
        let source = "a".as_bytes();
        let mut parser = Parser::new(source, "test");
        let expression = parser.parse_expression().unwrap();
        assert_eq!(expression, ast::Expression::Variable("a".to_string()));
    }

    #[test]
    fn literal_string_expression() {
        let source = "'test'".as_bytes();
        let mut parser = Parser::new(source, "test");
        let expression = parser.parse_expression().unwrap();
        assert_eq!(expression, ast::Expression::LiteralString("test".to_string()));
    }

    #[test]
    fn literal_nil_expression() {
        let source = "nil".as_bytes();
        let mut parser = Parser::new(source, "test");
        let expression = parser.parse_expression().unwrap();
        assert_eq!(expression, ast::Expression::LiteralNil);
    }

    #[test]
    fn literal_boolean_expression() {
        let source = "true || false".as_bytes();
        let mut parser = Parser::new(source, "test");
        let expression = parser.parse_expression().unwrap();
        assert_eq!(expression, ast::Expression::BinaryMessage {
            message: "||".to_string(),
            left: Box::new(ast::Expression::LiteralBoolean(true)),
            right: Box::new(ast::Expression::LiteralBoolean(false)),
        });
    }

    #[test]
    fn literal_symbol_expression() {
        let source = "#test #'test-case' #run:with:".as_bytes();
        let mut parser = Parser::new(source, "test");
        let expression = parser.parse_expression().unwrap();
        assert_eq!(expression, ast::Expression::LiteralSymbol("test".to_string()));
        let expression = parser.parse_expression().unwrap();
        assert_eq!(expression, ast::Expression::LiteralSymbol("test-case".to_string()));
        let expression = parser.parse_expression().unwrap();
        assert_eq!(expression, ast::Expression::LiteralSymbol("run:with:".to_string()));
    }

    #[test]
    fn literal_integer_expression() {
        let source = "1".as_bytes();
        let mut parser = Parser::new(source, "test");
        let expression = parser.parse_expression().unwrap();
        assert_eq!(expression, ast::Expression::LiteralInteger(1));
    }

    #[test]
    fn literal_negative_integer_expression() {
        let source = "-1".as_bytes();
        let mut parser = Parser::new(source, "test");
        let expression = parser.parse_expression().unwrap();
        assert_eq!(expression, ast::Expression::LiteralInteger(-1));
    }

    #[test]
    fn literal_negative_double_expression() {
        let source = "-3.14".as_bytes();
        let mut parser = Parser::new(source, "test");
        let expression = parser.parse_expression().unwrap();
        assert_eq!(expression, ast::Expression::LiteralDouble(-3.14));
    }

    #[test]
    fn literal_double_expression() {
        let source = "3.14".as_bytes();
        let mut parser = Parser::new(source, "test");
        let expression = parser.parse_expression().unwrap();
        assert_eq!(expression, ast::Expression::LiteralDouble(3.14));
    }

    #[test]
    fn unary_message_expression() {
        let source = "1 println".as_bytes();
        let mut parser = Parser::new(source, "test");
        let expression = parser.parse_expression().unwrap();
        assert_eq!(expression, ast::Expression::UnaryMessage {
            message: "println".to_string(),
            receiver: Box::new(ast::Expression::LiteralInteger(1)),
        });
    }

    #[test]
    fn multiple_unary_messages() {
        let source = "1 test println".as_bytes();
        let mut parser = Parser::new(source, "test");
        let expression = parser.parse_expression().unwrap();
        assert_eq!(expression, ast::Expression::UnaryMessage {
            message: "println".to_string(),
            receiver: Box::new(ast::Expression::UnaryMessage {
                message: "test".to_string(),
                receiver: Box::new(ast::Expression::LiteralInteger(1)),
            }),
        });
    }

    #[test]
    fn keyword_message_expression() {
        let source = "1 with: a and: b".as_bytes();
        let mut parser = Parser::new(source, "test");
        let expression = parser.parse_expression().unwrap();
        assert_eq!(expression, ast::Expression::KeywordMessage {
            message: "with:and:".to_string(),
            parameters: vec![
                ast::Expression::Variable("a".to_string()),
                ast::Expression::Variable("b".to_string()),
            ],
            receiver: Box::new(ast::Expression::LiteralInteger(1)),
        });
    }

    #[test]
    fn complex_keyword_message_expression() {
        let source = "1 with: a length and: 1 + 2".as_bytes();
        let mut parser = Parser::new(source, "test");
        let expression = parser.parse_expression().unwrap();
        println!("expression: {:#?}", expression);
        assert_eq!(expression, ast::Expression::KeywordMessage {
            message: "with:and:".to_string(),
            parameters: vec![
                ast::Expression::UnaryMessage {
                    message: "length".to_string(),
                    receiver: Box::new(ast::Expression::Variable("a".to_string())),
                },
                ast::Expression::BinaryMessage {
                    message: "+".to_string(),
                    left: Box::new(ast::Expression::LiteralInteger(1)),
                    right: Box::new(ast::Expression::LiteralInteger(2)),
                },
            ],
            receiver: Box::new(ast::Expression::LiteralInteger(1)),
        });
    }

    #[test]
    fn binary_message_expression() {
        let source = "1 + 2".as_bytes();
        let mut parser = Parser::new(source, "test");
        let expression = parser.parse_expression().unwrap();
        assert_eq!(expression, ast::Expression::BinaryMessage {
            message: "+".to_string(),
            left: Box::new(ast::Expression::LiteralInteger(1)),
            right: Box::new(ast::Expression::LiteralInteger(2)),
        });
    }

    #[test]
    fn operator_sequence_expression() {
        let source = "1 <= 2".as_bytes();
        let mut parser = Parser::new(source, "test");
        let expression = parser.parse_expression().unwrap();
        assert_eq!(expression, ast::Expression::BinaryMessage {
            message: "<=".to_string(),
            left: Box::new(ast::Expression::LiteralInteger(1)),
            right: Box::new(ast::Expression::LiteralInteger(2)),
        });
    }

    #[test]
    fn nested_terms() {
        let source = "1 + (2 - 1)".as_bytes();
        let mut parser = Parser::new(source, "test");
        let expression = parser.parse_expression().unwrap();
        println!("expression: {:#?}", expression);
        assert_eq!(expression, ast::Expression::BinaryMessage {
            message: "+".to_string(),
            left: Box::new(ast::Expression::LiteralInteger(1)),
            right: Box::new(ast::Expression::BinaryMessage {
                message: "-".to_string(),
                left: Box::new(ast::Expression::LiteralInteger(2)),
                right: Box::new(ast::Expression::LiteralInteger(1)),
            }),
        });
    }

    #[test]
    fn unary_message_binds_higher() {
        let source = "1 test + 2".as_bytes();
        let mut parser = Parser::new(source, "test");
        let expression = parser.parse_expression().unwrap();
        assert_eq!(expression, ast::Expression::BinaryMessage {
            message: "+".to_string(),
            left: Box::new(ast::Expression::UnaryMessage {
                receiver: Box::new(ast::Expression::LiteralInteger(1)),
                message: "test".to_string(),
            }),
            right: Box::new(ast::Expression::LiteralInteger(2)),
        });
    }

    #[test]
    fn superclass_parsing() {
        let source = "Hello = Test ()".as_bytes();
        let mut parser = Parser::new(source, "test");
        let class = parser.parse_class().unwrap();
        assert_eq!(class.superclass, "Test");
        assert_eq!(class.name, "Hello");
    }

    #[test]
    fn method_with_locals() {
        let source = "
        test = ( |a b|
            a println
        )
        ".as_bytes();
        let mut parser = Parser::new(source, "test");
        let (_, method) = parser.parse_method().unwrap();
        assert_eq!(method, ast::Method::Native {
            name: "test".to_string(),
            parameters: vec![],
            locals: vec!["a".to_string(), "b".to_string()],
            body: vec![
                ast::Expression::UnaryMessage {
                    receiver: Box::new(ast::Expression::Variable("a".to_string())),
                    message: "println".to_string(),
                },
            ],
        });
    }

    #[test]
    fn method_with_multiple_statements() {
        let source = "
        test = ( |a b|
            a println.
            b println.
        )
        ".as_bytes();
        let mut parser = Parser::new(source, "test");
        let (_, method) = parser.parse_method().unwrap();
        assert_eq!(method, ast::Method::Native {
            name: "test".to_string(),
            parameters: vec![],
            locals: vec!["a".to_string(), "b".to_string()],
            body: vec![
                ast::Expression::UnaryMessage {
                    receiver: Box::new(ast::Expression::Variable("a".to_string())),
                    message: "println".to_string(),
                },
                ast::Expression::UnaryMessage {
                    receiver: Box::new(ast::Expression::Variable("b".to_string())),
                    message: "println".to_string(),
                },
            ],
        });
    }

    #[test]
    fn method_with_parameters() {
        let source = "
        test: a with: b = (
            a println
        )
        ".as_bytes();
        let mut parser = Parser::new(source, "test");
        let (_, method) = parser.parse_method().unwrap();
        assert_eq!(method, ast::Method::Native {
            name: "test:with:".to_string(),
            parameters: vec!["a".to_string(), "b".to_string()],
            locals: vec![],
            body: vec![
                ast::Expression::UnaryMessage {
                    receiver: Box::new(ast::Expression::Variable("a".to_string())),
                    message: "println".to_string(),
                },
            ],
        });
    }

    #[test]
    fn method_with_exit() {
        let source = "
        test = (
            ^ 1 + 1.
        )
        ".as_bytes();
        let mut parser = Parser::new(source, "test");
        let (_, method) = parser.parse_method().unwrap();
        assert_eq!(method, ast::Method::Native {
            name: "test".to_string(),
            parameters: vec![],
            locals: vec![],
            body: vec![
                ast::Expression::Return(Box::new(
                    ast::Expression::BinaryMessage {
                        message: "+".to_string(),
                        left: Box::new(ast::Expression::LiteralInteger(1)),
                        right: Box::new(ast::Expression::LiteralInteger(1)),
                    },
                )),
            ],
        });
    }
}

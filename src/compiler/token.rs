#[derive(Clone, Debug, PartialEq)]
pub enum Symbol {
    And,
    Assign,
    At,
    Colon,
    Comma,
    Divide,
    Double,
    EndBlock,
    EndTerm,
    Equal,
    Exit,
    Identifier,
    Integer,
    Keyword,
    KeywordSequence,
    Less,
    Minus,
    Modulus,
    More,
    NewBlock,
    NewTerm,
    None,
    Not,
    OperatorSequence,
    Or,
    Percent,
    Period,
    Plus,
    Pound,
    Primitive,
    Separator,
    Star,
    String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token(pub Symbol, pub Option<String>);

impl From<Symbol> for Token {
    fn from(symbol: Symbol) -> Token {
        Token(symbol, None)
    }
}

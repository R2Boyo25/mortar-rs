#[derive(PartialEq, Debug)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub index: usize,
}

#[derive(PartialEq, Debug)]
pub enum TokenKind {
    Identifier,
    Integer,
    Float,

    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,

    Plus,
    Hyphen,
    ForwardSlash,
    Asterisk,
    Equal,
    Less,
    Greater,
    Bang,

    EqualEqual,
    LessLess,
    GreaterGreater,
    PlusPlus,
    HyphenHyphen,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq, Debug)]
pub struct Token {
    pub location: Location,
    pub kind: TokenKind,
    pub contents: String,
}

impl Token {
    pub fn new(location: Location, kind: TokenKind, contents: &str) -> Self {
        Self {
            location,
            kind,
            contents: contents.to_owned(),
        }
    }
}

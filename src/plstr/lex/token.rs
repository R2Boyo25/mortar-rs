#[derive(PartialEq, Debug)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub index: usize,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.line + 1, self.column + 1)
    }
}

#[derive(PartialEq, Debug)]
pub enum TokenKind {
    Identifier,
    Integer,
    Float,
    String,

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
    Comma,
    Colon,

    BangEqual,
    EqualEqual,
    LessLess,
    GreaterGreater,
    PlusPlus,
    HyphenHyphen,
    ColonColon,
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

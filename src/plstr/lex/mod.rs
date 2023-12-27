pub mod token;
use std::iter::Peekable;
use unicode_segmentation::UnicodeSegmentation;

use token::{Location, Token, TokenKind};

pub struct Lexer {
    body: String,
    iter: Peekable<Box<dyn Iterator<Item = char>>>,
    line: usize,
    col: usize,
    idx: usize,
}

impl Lexer {
    pub fn new(body: &'static str) -> Self {
        Self {
            body: body.to_owned(),
            iter: (Box::new(body.chars()) as Box<dyn Iterator<Item = char>>).peekable(),
            line: 0,
            col: 0,
            idx: 0,
        }
    }

    fn consume_whitespace(&mut self) {
        while let Some(c) = self.iter.peek() {
            if !c.is_whitespace() {
                break;
            }

            let c = c.to_owned();

            if c == '\n' {
                self.new_line(c);
            }

            self.iter.next();
            self.inc_loc(c);
        }
    }

    pub fn get_location(&self) -> Location {
        Location {
            line: self.line,
            column: self.col,
            index: self.idx,
        }
    }

    fn new_line(&mut self, c: char) {
        self.line += 1;
        self.idx += c.len_utf8();
        self.col = 0;
    }

    fn inc_loc(&mut self, c: char) {
        self.col += 1;
        self.idx += c.len_utf8();
    }

    fn new_token(&self, kind: TokenKind, start_idx: usize) -> Token {
        Token::new(self.get_location(), kind, &self.body[start_idx..self.idx])
    }

    fn single_token(&self, kind: TokenKind) -> Option<Token> {
        Some(self.new_token(kind, self.idx - 1))
    }

    fn rep2(&mut self, c: char, first: TokenKind, second: TokenKind) -> Option<Token> {
        if self.iter.peek() == Some(&c) {
            self.iter.next();
            self.inc_loc(c);
            self.single_token(second)
        } else {
            self.single_token(first)
        }
    }

    fn two(&mut self, matcher: fn (char) -> Option<TokenKind>, other: TokenKind) -> Option<Token> {
        if self.iter.peek() == None {
            return self.single_token(other);
        }
        
        let c = *self.iter.peek().unwrap();
        if let Some(typ) = matcher(c) {
            self.iter.next();
            self.inc_loc(c);
            self.single_token(typ)
        } else {
            self.single_token(other)
        }
    }
}

fn is_emoji(c: char) -> bool {
    emojis::get(&c.to_string()).is_some()
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.consume_whitespace();
        let start_idx = self.idx;

        if let Some(c) = self.iter.next() {
            self.inc_loc(c);

            match c {
                '{' => self.single_token(TokenKind::OpenBrace),
                '}' => self.single_token(TokenKind::CloseBrace),
                '(' => self.single_token(TokenKind::OpenParen),
                ')' => self.single_token(TokenKind::CloseParen),
                '[' => self.single_token(TokenKind::OpenBracket),
                ']' => self.single_token(TokenKind::CloseBracket),
                '+' => self.rep2('+', TokenKind::Plus, TokenKind::PlusPlus),
                '-' => self.rep2('-', TokenKind::Hyphen, TokenKind::HyphenHyphen),
                '/' => self.single_token(TokenKind::ForwardSlash),
                '*' => self.single_token(TokenKind::Asterisk),
                '=' => self.rep2('=', TokenKind::Equal, TokenKind::EqualEqual),
                '<' => self.rep2('<', TokenKind::Less, TokenKind::LessLess),
                '>' => self.rep2('>', TokenKind::Greater, TokenKind::GreaterGreater),
                '!' => self.single_token(TokenKind::Bang),
                _ => {
                    if c.is_alphabetic() || c == '_' || is_emoji(c) {
                        // Identifier

                        while let Some(c) = self.iter.peek() {
                            let c = c.to_owned();
                            if c.is_alphanumeric() || c == '_'  || is_emoji(c) {
                                self.iter.next();
                                self.inc_loc(c);
                            } else {
                                break;
                            }
                        }

                        return Some(self.new_token(TokenKind::Identifier, start_idx));
                    } else if c.is_numeric() {
                        // Number
                        let mut is_float = false;

                        while let Some(c) = self.iter.peek() {
                            let c = c.clone();

                            if c.is_numeric() {
                                self.iter.next();
                                self.inc_loc(c);
                            } else if c == '.' {
                                if is_float {
                                    panic!("Float cannot have multiple decimal points.");
                                }

                                is_float = true;
                                self.inc_loc(c);
                                self.iter.next();
                            } else {
                                break;
                            }
                        }

                        return Some(self.new_token(
                            if is_float {
                                TokenKind::Float
                            } else {
                                TokenKind::Integer
                            },
                            start_idx,
                        ));
                    }

                    panic!("Unexpected character '{c}'");
                }
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::token::TokenKind;
    use super::Lexer;

    #[test]
    fn a() {
        let mut lexer = Lexer::new("   abc  23 2.3");
        let mut token = lexer.next().unwrap();

        assert_eq!(token.contents, "abc");
        assert_eq!(token.kind, TokenKind::Identifier);

        token = lexer.next().unwrap();

        assert_eq!(token.contents, "23");
        assert_eq!(token.kind, TokenKind::Integer);

        token = lexer.next().unwrap();

        assert_eq!(token.contents, "2.3");
        assert_eq!(token.kind, TokenKind::Float);
    }

    #[test]
    fn unicode() {
        let mut lexer = Lexer::new("a😭bc\r\na");
        let mut token = lexer.next().unwrap();

        assert_eq!(token.contents, "a😭bc");
        assert_eq!(token.kind, TokenKind::Identifier);

        println!("{:?}", lexer.body.chars().collect::<Vec<_>>());

        token = lexer.next().unwrap();

        assert_eq!(token.contents, "a");
        assert_eq!(token.kind, TokenKind::Identifier);
    }
}

pub mod token;
use std::iter::Peekable;
use unicode_segmentation::UnicodeSegmentation;

use token::{Location, Token, TokenKind};

pub struct Lexer {
    body: String,
    iter: Peekable<Box<dyn Iterator<Item = &'static str>>>,
    line: usize,
    col: usize,
    idx: usize,
}

impl Lexer {
    pub fn new(body: &'static str) -> Self {
        Self {
            body: body.to_owned(),
            iter: (Box::new(body.graphemes(true)) as Box<dyn Iterator<Item = &str>>).peekable(),
            line: 0,
            col: 0,
            idx: 0,
        }
    }

    fn consume_whitespace(&mut self) {
        while let Some(c) = self.iter.peek() {
            if !(vec![" ", "\t", "\n", "\r", "\r\n"].contains(c)) {
                break;
            }

            let c = c.to_owned();

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

    fn new_line(&mut self, c: &str) {
        self.line += 1;
        self.idx += c.len();
        self.col = 0;
    }

    fn inc_loc(&mut self, c: &str) {
        if vec!["\n", "\r\n"].contains(&c) {
            self.new_line(c);
            return;
        }

        self.col += 1;
        self.idx += c.len();
    }

    fn new_token(&self, kind: TokenKind, start_idx: usize) -> Token {
        Token::new(self.get_location(), kind, &self.body[start_idx..self.idx])
    }

    fn single_token(&self, kind: TokenKind) -> Option<Token> {
        Some(self.new_token(kind, self.idx - 1))
    }

    fn rep2(&mut self, c: &str, first: TokenKind, second: TokenKind) -> Option<Token> {
        if self.iter.peek() == Some(&c) {
            self.iter.next();
            self.inc_loc(c);
            self.single_token(second)
        } else {
            self.single_token(first)
        }
    }

    fn two(&mut self, matcher: fn(&str) -> Option<TokenKind>, other: TokenKind) -> Option<Token> {
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

fn is_emoji(c: &str) -> bool {
    emojis::get(c).is_some()
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.consume_whitespace();
        let start_idx = self.idx;
        
        if let Some(c) = self.iter.next() {
            self.inc_loc(c);

            match c {
                "{" => self.single_token(TokenKind::OpenBrace),
                "}" => self.single_token(TokenKind::CloseBrace),
                "(" => self.single_token(TokenKind::OpenParen),
                ")" => self.single_token(TokenKind::CloseParen),
                "[" => self.single_token(TokenKind::OpenBracket),
                "]" => self.single_token(TokenKind::CloseBracket),
                "+" => self.rep2("+", TokenKind::Plus, TokenKind::PlusPlus),
                "-" => self.rep2("-", TokenKind::Hyphen, TokenKind::HyphenHyphen),
                "/" => self.single_token(TokenKind::ForwardSlash),
                "*" => self.single_token(TokenKind::Asterisk),
                "=" => self.rep2("=", TokenKind::Equal, TokenKind::EqualEqual),
                "<" => self.rep2("<", TokenKind::Less, TokenKind::LessLess),
                ">" => self.rep2(">", TokenKind::Greater, TokenKind::GreaterGreater),
                "!" => self.single_token(TokenKind::Bang),
                _ => {
                    if c.contains(char::is_alphabetic) || c == "_" || is_emoji(c) {
                        // Identifier

                        while let Some(c) = self.iter.peek() {
                            let c = c.to_owned();
                            if c.contains(char::is_alphanumeric) || c == "_" || is_emoji(c) {
                                self.iter.next();
                                self.inc_loc(c);
                            } else {
                                break;
                            }
                        }

                        return Some(self.new_token(TokenKind::Identifier, start_idx));
                    } else if c.contains(char::is_numeric) {
                        // Number
                        let mut is_float = false;

                        while let Some(c) = self.iter.peek() {
                            let c = c.to_owned();

                            if c.contains(char::is_numeric) {
                                self.iter.next();
                                self.inc_loc(c);
                            } else if c == "." {
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
    fn basic_types() {
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
        let mut lexer = Lexer::new("a😭bc\r\nd");
        let mut token = lexer.next().unwrap();

        assert_eq!(token.contents, "a😭bc");
        assert_eq!(token.kind, TokenKind::Identifier);

        token = lexer.next().unwrap();

        assert_eq!(token.contents, "d");
        assert_eq!(token.kind, TokenKind::Identifier);
    }
}

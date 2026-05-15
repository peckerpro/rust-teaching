use rt_common::{
    diagnostic::DiagnosticBag,
    span::{SourceFile, Span},
    token::{Token, TokenKind},
};
use std::sync::Arc;

use crate::cursor::Cursor;

pub struct Lexer<'a> {
    cursor: Cursor<'a>,
    pub diagnostics: DiagnosticBag,
    eof_reached: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str, file: Arc<SourceFile>) -> Self {
        Lexer {
            cursor: Cursor::new(src, file),
            diagnostics: DiagnosticBag::new(),
            eof_reached: false,
        }
    }

    fn span(&self, lo: usize) -> Span {
        Span::new(BytePos(lo as u32), self.cursor.pos())
    }

    fn skip_whitespace(&mut self) {
        while !self.cursor.is_eof() {
            let b = self.cursor.peek().unwrap();
            if b.is_ascii_whitespace() {
                self.cursor.advance();
            } else {
                break;
            }
        }
    }

    fn lex_ident(&mut self, start: usize) -> Token {
        self.cursor.eat_while(|b| b.is_ascii_alphanumeric() || b == b'_');
        let ident = self.cursor.slice(BytePos(start as u32), self.cursor.pos()).to_string();
        let kind = TokenKind::from_keyword(&ident).unwrap_or(TokenKind::Ident);
        Token::new(kind, self.span(start))
    }

    fn lex_number(&mut self, start: usize) -> Token {
        if self.cursor.peek() == Some(b'x') {
            self.cursor.advance();
            self.cursor.eat_while(|b| b.is_ascii_hexdigit() || b == b'_');
        } else if self.cursor.peek() == Some(b'o') {
            self.cursor.advance();
            self.cursor.eat_while(|b| (b'0'..=b'7').contains(&b) || b == b'_');
        } else if self.cursor.peek() == Some(b'b') {
            self.cursor.advance();
            self.cursor.eat_while(|b| b == b'0' || b == b'1' || b == b'_');
        } else {
            self.cursor.eat_while(|b| b.is_ascii_digit() || b == b'_');
        }

        let is_float = self.cursor.peek() == Some(b'.')
            && self.cursor.peek_n(1) != Some(b'.');

        if is_float {
            self.cursor.advance();
            self.cursor.eat_while(|b| b.is_ascii_digit() || b == b'_');
            if self.cursor.peek().map_or(false, |b| b == b'e' || b == b'E') {
                self.cursor.advance();
                let _ = self.cursor.eat_byte(b'+') || self.cursor.eat_byte(b'-');
                self.cursor.eat_while(|b| b.is_ascii_digit() || b == b'_');
            }
            // parse float suffix (f32, f64)
            self.cursor.eat_while(|b| b.is_ascii_alphanumeric() || b == b'_');
            Token::new(TokenKind::Float, self.span(start))
        } else {
            // parse integer suffix (i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, isize, usize)
            self.cursor.eat_while(|b| b.is_ascii_alphanumeric() || b == b'_');
            Token::new(TokenKind::Integer, self.span(start))
        }
    }

    fn lex_char_or_byte(&mut self, start: usize, is_byte: bool) -> Token {
        self.cursor.advance();

        if self.cursor.eat_byte(b'\\') {
            self.cursor.advance();
        } else if !self.cursor.is_eof() {
            self.cursor.advance();
        }

        let _ = self.cursor.eat_byte(b'\'');

        if is_byte {
            Token::new(TokenKind::Byte, self.span(start))
        } else {
            Token::new(TokenKind::Char, self.span(start))
        }
    }

    fn lex_string(&mut self, start: usize, is_byte: bool) -> Token {
        self.cursor.advance();

        loop {
            match self.cursor.peek() {
                Some(b'\"') => {
                    self.cursor.advance();
                    break;
                }
                Some(b'\\') => {
                    self.cursor.advance();
                    self.cursor.advance();
                }
                Some(_) => {
                    self.cursor.advance();
                }
                None => {
                    let file = self.cursor.file.clone();
                    self.diagnostics.error(
                        "unterminated string literal",
                        self.span(start),
                        file,
                    );
                    break;
                }
            }
        }

        if is_byte {
            Token::new(TokenKind::ByteString, self.span(start))
        } else {
            Token::new(TokenKind::String, self.span(start))
        }
    }

    fn lex_line_comment(&mut self, start: usize) -> Token {
        let is_doc = self.cursor.peek().map_or(false, |b| {
            b == b'/' || b == b'!' || b.is_ascii_alphabetic()
        });

        loop {
            match self.cursor.peek() {
                Some(b'\n') | None => break,
                Some(_) => {
                    self.cursor.advance();
                }
            }
        }

        if is_doc {
            Token::new(TokenKind::DocComment, self.span(start))
        } else {
            Token::new(TokenKind::LineComment, self.span(start))
        }
    }

    fn lex_block_comment(&mut self, start: usize) -> Token {
        let is_doc = self.cursor.peek().map_or(false, |b| b == b'*' || b == b'!');

        let mut depth = 1u32;
        loop {
            match self.cursor.advance() {
                Some(b'*') if self.cursor.peek() == Some(b'/') => {
                    self.cursor.advance();
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                Some(b'/') if self.cursor.peek() == Some(b'*') => {
                    self.cursor.advance();
                    depth += 1;
                }
                None => {
                    let file = self.cursor.file.clone();
                    self.diagnostics.error(
                        "unterminated block comment",
                        self.span(start),
                        file,
                    );
                    break;
                }
                Some(_) => {}
            }
        }

        if is_doc {
            Token::new(TokenKind::DocComment, self.span(start))
        } else {
            Token::new(TokenKind::BlockComment, self.span(start))
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        loop {
            if self.eof_reached {
                return None;
            }

            self.skip_whitespace();
            let start = self.cursor.pos().to_usize();

            let b = match self.cursor.advance() {
                Some(b) => b,
                None => {
                    self.eof_reached = true;
                    return None;
                }
            };

            if b == b'/' && self.cursor.peek() == Some(b'/') {
                self.cursor.advance();
                let _ = self.lex_line_comment(start);
                continue;
            }
            if b == b'/' && self.cursor.peek() == Some(b'*') {
                self.cursor.advance();
                let _ = self.lex_block_comment(start);
                continue;
            }
            if b == b'#' && self.cursor.peek() == Some(b'!') {
                self.cursor.advance();
                let _ = self.lex_line_comment(start);
                continue;
            }

            let token = match b {
                b'_' if !self
                    .cursor
                    .peek()
                    .map_or(false, |c| c.is_ascii_alphanumeric()) =>
                {
                    Token::new(TokenKind::Underscore, self.span(start))
                }

                c if c.is_ascii_alphabetic() || c == b'_' => {
                    if c == b'b'
                        && (self.cursor.peek() == Some(b'\'')
                            || self.cursor.peek() == Some(b'"'))
                    {
                        if self.cursor.peek() == Some(b'\'') {
                            self.lex_char_or_byte(start, true)
                        } else {
                            self.lex_string(start, true)
                        }
                    } else {
                        self.lex_ident(start)
                    }
                }

                c if c.is_ascii_digit() => self.lex_number(start),

                b'\'' => self.lex_char_or_byte(start, false),
                b'"' => self.lex_string(start, false),

                b'(' => Token::new(TokenKind::LParen, self.span(start)),
                b')' => Token::new(TokenKind::RParen, self.span(start)),
                b'{' => Token::new(TokenKind::LBrace, self.span(start)),
                b'}' => Token::new(TokenKind::RBrace, self.span(start)),
                b'[' => Token::new(TokenKind::LBracket, self.span(start)),
                b']' => Token::new(TokenKind::RBracket, self.span(start)),
                b',' => Token::new(TokenKind::Comma, self.span(start)),
                b';' => Token::new(TokenKind::Semi, self.span(start)),
                b':' => {
                    if self.cursor.eat_byte(b':') {
                        Token::new(TokenKind::PathSep, self.span(start))
                    } else {
                        Token::new(TokenKind::Colon, self.span(start))
                    }
                }
                b'@' => Token::new(TokenKind::At, self.span(start)),
                b'#' => {
                    Token::new(TokenKind::Pound, self.span(start))
                }
                b'$' => Token::new(TokenKind::Dollar, self.span(start)),
                b'?' => Token::new(TokenKind::Question, self.span(start)),
                b'.' => {
                    if self.cursor.eat_byte(b'.') {
                        if self.cursor.eat_byte(b'=') {
                            Token::new(TokenKind::DotDotEq, self.span(start))
                        } else {
                            Token::new(TokenKind::DotDot, self.span(start))
                        }
                    } else if self
                        .cursor
                        .peek()
                        .map_or(false, |c| c.is_ascii_digit())
                    {
                        self.lex_number(start)
                    } else {
                        Token::new(TokenKind::Dot, self.span(start))
                    }
                }
                b'&' => {
                    if self.cursor.eat_byte(b'&') {
                        Token::new(TokenKind::AndAnd, self.span(start))
                    } else if self.cursor.eat_byte(b'=') {
                        Token::new(TokenKind::AndEq, self.span(start))
                    } else {
                        Token::new(TokenKind::And, self.span(start))
                    }
                }
                b'|' => {
                    if self.cursor.eat_byte(b'|') {
                        Token::new(TokenKind::OrOr, self.span(start))
                    } else if self.cursor.eat_byte(b'=') {
                        Token::new(TokenKind::OrEq, self.span(start))
                    } else {
                        Token::new(TokenKind::Or, self.span(start))
                    }
                }
                b'^' => {
                    if self.cursor.eat_byte(b'=') {
                        Token::new(TokenKind::CaretEq, self.span(start))
                    } else {
                        Token::new(TokenKind::Caret, self.span(start))
                    }
                }
                b'!' => {
                    if self.cursor.eat_byte(b'=') {
                        Token::new(TokenKind::Ne, self.span(start))
                    } else {
                        Token::new(TokenKind::Not, self.span(start))
                    }
                }
                b'=' => {
                    if self.cursor.eat_byte(b'=') {
                        Token::new(TokenKind::EqEq, self.span(start))
                    } else if self.cursor.eat_byte(b'>') {
                        Token::new(TokenKind::FatArrow, self.span(start))
                    } else {
                        Token::new(TokenKind::Eq, self.span(start))
                    }
                }
                b'<' => {
                    if self.cursor.eat_byte(b'<') {
                        if self.cursor.eat_byte(b'=') {
                            Token::new(TokenKind::ShlEq, self.span(start))
                        } else {
                            Token::new(TokenKind::Shl, self.span(start))
                        }
                    } else if self.cursor.eat_byte(b'=') {
                        Token::new(TokenKind::Le, self.span(start))
                    } else {
                        Token::new(TokenKind::Lt, self.span(start))
                    }
                }
                b'>' => {
                    if self.cursor.eat_byte(b'>') {
                        if self.cursor.eat_byte(b'=') {
                            Token::new(TokenKind::ShrEq, self.span(start))
                        } else {
                            Token::new(TokenKind::Shr, self.span(start))
                        }
                    } else if self.cursor.eat_byte(b'=') {
                        Token::new(TokenKind::Ge, self.span(start))
                    } else {
                        Token::new(TokenKind::Gt, self.span(start))
                    }
                }
                b'+' => {
                    if self.cursor.eat_byte(b'=') {
                        Token::new(TokenKind::PlusEq, self.span(start))
                    } else {
                        Token::new(TokenKind::Plus, self.span(start))
                    }
                }
                b'-' => {
                    if self.cursor.eat_byte(b'>') {
                        Token::new(TokenKind::RArrow, self.span(start))
                    } else if self.cursor.eat_byte(b'=') {
                        Token::new(TokenKind::MinusEq, self.span(start))
                    } else {
                        Token::new(TokenKind::Minus, self.span(start))
                    }
                }
                b'*' => {
                    if self.cursor.eat_byte(b'=') {
                        Token::new(TokenKind::StarEq, self.span(start))
                    } else {
                        Token::new(TokenKind::Star, self.span(start))
                    }
                }
                b'/' => {
                    if self.cursor.eat_byte(b'=') {
                        Token::new(TokenKind::SlashEq, self.span(start))
                    } else {
                        Token::new(TokenKind::Slash, self.span(start))
                    }
                }
                b'%' => {
                    if self.cursor.eat_byte(b'=') {
                        Token::new(TokenKind::PercentEq, self.span(start))
                    } else {
                        Token::new(TokenKind::Percent, self.span(start))
                    }
                }

                _ => Token::new(TokenKind::Unknown, self.span(start)),
            };

            return Some(token);
        }
    }
}

use rt_common::span::BytePos;

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(source: &str) -> Vec<TokenKind> {
        let file = Arc::new(SourceFile::new("test.rs".to_string(), source.to_string()));
        Lexer::new(source, file)
            .filter(|t| t.kind != TokenKind::Eof)
            .map(|t| t.kind)
            .collect()
    }

    #[test]
    fn test_keywords() {
        let tokens = lex("let mut fn if else loop while for in match return");
        assert_eq!(tokens[0], TokenKind::KwLet);
        assert_eq!(tokens[1], TokenKind::KwMut);
        assert_eq!(tokens[2], TokenKind::KwFn);
        assert_eq!(tokens[3], TokenKind::KwIf);
        assert_eq!(tokens[4], TokenKind::KwElse);
        assert_eq!(tokens[5], TokenKind::KwLoop);
        assert_eq!(tokens[6], TokenKind::KwWhile);
        assert_eq!(tokens[7], TokenKind::KwFor);
        assert_eq!(tokens[8], TokenKind::KwIn);
        assert_eq!(tokens[9], TokenKind::KwMatch);
        assert_eq!(tokens[10], TokenKind::KwReturn);
    }

    #[test]
    fn test_operators() {
        let tokens = lex("+ - * / % == != < > <= >= && || ! =");
        assert_eq!(tokens, vec![
            TokenKind::Plus, TokenKind::Minus, TokenKind::Star, TokenKind::Slash,
            TokenKind::Percent, TokenKind::EqEq, TokenKind::Ne, TokenKind::Lt,
            TokenKind::Gt, TokenKind::Le, TokenKind::Ge, TokenKind::AndAnd,
            TokenKind::OrOr, TokenKind::Not, TokenKind::Eq,
        ]);
    }

    #[test]
    fn test_delimiters() {
        let tokens = lex("() {} [] , ; : :: -> => . .. ..=");
        assert_eq!(tokens, vec![
            TokenKind::LParen, TokenKind::RParen,
            TokenKind::LBrace, TokenKind::RBrace,
            TokenKind::LBracket, TokenKind::RBracket,
            TokenKind::Comma, TokenKind::Semi, TokenKind::Colon,
            TokenKind::PathSep, TokenKind::RArrow, TokenKind::FatArrow,
            TokenKind::Dot, TokenKind::DotDot, TokenKind::DotDotEq,
        ]);
    }

    #[test]
    fn test_literals() {
        let tokens = lex("42 3.14 true false 'a' \"hello\" b\"world\"");
        assert_eq!(tokens, vec![
            TokenKind::Integer, TokenKind::Float,
            TokenKind::KwTrue, TokenKind::KwFalse,
            TokenKind::Char,
            TokenKind::String,
            TokenKind::ByteString,
        ]);
    }

    #[test]
    fn test_comments_are_skipped() {
        let tokens = lex("// line comment\nx /* block */ y");
        assert_eq!(tokens, vec![
            TokenKind::Ident, TokenKind::Ident,
        ]);
    }

    #[test]
    fn test_compound_assign() {
        let tokens = lex("+= -= *= /= %= &= |= ^= <<= >>=");
        assert_eq!(tokens, vec![
            TokenKind::PlusEq, TokenKind::MinusEq, TokenKind::StarEq,
            TokenKind::SlashEq, TokenKind::PercentEq, TokenKind::AndEq,
            TokenKind::OrEq, TokenKind::CaretEq, TokenKind::ShlEq,
            TokenKind::ShrEq,
        ]);
    }

    #[test]
    fn test_path() {
        let tokens = lex("std::collections::HashMap");
        assert_eq!(tokens, vec![
            TokenKind::Ident, TokenKind::PathSep,
            TokenKind::Ident, TokenKind::PathSep,
            TokenKind::Ident,
        ]);
    }
}

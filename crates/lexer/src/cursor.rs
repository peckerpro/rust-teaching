use rt_common::span::{BytePos, SourceFile};
use std::sync::Arc;

pub struct Cursor<'a> {
    pub(crate) src: &'a str,
    pos: usize,
    pub file: Arc<SourceFile>,
}

impl<'a> Cursor<'a> {
    pub fn new(src: &'a str, file: Arc<SourceFile>) -> Self {
        Cursor { src, pos: 0, file }
    }

    pub fn pos(&self) -> BytePos {
        BytePos(self.pos as u32)
    }

    pub fn is_eof(&self) -> bool {
        self.pos >= self.src.len()
    }

    pub fn peek(&self) -> Option<u8> {
        self.src.as_bytes().get(self.pos).copied()
    }

    pub fn peek_n(&self, n: usize) -> Option<u8> {
        self.src.as_bytes().get(self.pos + n).copied()
    }

    pub fn advance(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.pos += 1;
        Some(b)
    }

    pub fn advance_char(&mut self) -> Option<u8> {
        let b = self.peek()?;
        let len = if b < 0x80 { 1 }
                  else if (b & 0xE0) == 0xC0 { 2 }
                  else if (b & 0xF0) == 0xE0 { 3 }
                  else if (b & 0xF8) == 0xF0 { 4 }
                  else { 1 }; // continuation bytes: treat as 1
        self.pos += len;
        Some(b)
    }

    pub fn advance_n(&mut self, n: usize) {
        self.pos = (self.pos + n).min(self.src.len());
    }

    pub fn eat_if(&mut self, f: impl Fn(u8) -> bool) -> bool {
        if let Some(b) = self.peek() {
            if f(b) {
                self.advance();
                return true;
            }
        }
        false
    }

    pub fn eat_byte(&mut self, byte: u8) -> bool {
        self.eat_if(|b| b == byte)
    }

    pub fn eat_str(&mut self, s: &str) -> bool {
        let len = s.len();
        if self.src[self.pos..].starts_with(s) {
            self.advance_n(len);
            true
        } else {
            false
        }
    }

    pub fn slice(&self, start: BytePos, end: BytePos) -> &'a str {
        &self.src[start.to_usize()..end.to_usize()]
    }

    pub fn eat_while(&mut self, f: impl Fn(u8) -> bool) {
        while !self.is_eof() && f(self.peek().unwrap()) {
            self.advance();
        }
    }
}

fn utf8_byte_width(b: u8) -> usize {
    if b < 0x80 { 1 }
    else if b < 0xC0 { 1 }
    else if b < 0xE0 { 2 }
    else if b < 0xF0 { 3 }
    else { 4 }
}

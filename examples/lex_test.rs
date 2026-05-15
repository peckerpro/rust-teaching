use rt_common::span::{SourceFile, BytePos, Span};
use rt_common::token::{Token, TokenKind};
use std::sync::Arc;

fn main() {
    let src = "let x = 42;";
    let file = Arc::new(SourceFile::new("test".into(), src.into()));
    let mut lexer = rt_lexer::Lexer::new(src, file);
    for _ in 0..10 {
        match lexer.next() {
            Some(tok) => println!("{:?}", tok.kind),
            None => break,
        }
    }
    println!("Done");
}

use rt_common::span::Span;
use super::expr::Expr;

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<super::node::Stmt>,
    pub expr: Option<Box<Expr>>,
    pub span: Span,
}

impl Block {
    pub fn new(stmts: Vec<super::node::Stmt>, expr: Option<Box<Expr>>, span: Span) -> Self {
        Block { stmts, expr, span }
    }
}

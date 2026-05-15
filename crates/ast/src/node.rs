use crate::{expr::Expr, item::Item, pattern::Pattern, ty::Ty};
use rt_common::span::Span;

#[derive(Debug, Clone)]
pub enum Node {
    Expr(Expr),
    Stmt(Stmt),
    Item(Item),
    Pattern(Pattern),
    Ty(Ty),
}

impl Node {
    pub fn span(&self) -> Span {
        match self {
            Node::Expr(e) => e.span(),
            Node::Stmt(s) => s.span(),
            Node::Item(i) => i.span(),
            Node::Pattern(p) => p.span(),
            Node::Ty(t) => t.span(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        pattern: Pattern,
        ty: Option<Ty>,
        init: Option<Expr>,
        span: Span,
    },
    Expr(Expr),
    Item(Item),
    Semi,
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Let { span, .. } => *span,
            Stmt::Expr(e) => e.span(),
            Stmt::Item(i) => i.span(),
            Stmt::Semi => Span::DUMMY,
        }
    }
}

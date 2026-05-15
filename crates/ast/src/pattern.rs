use rt_common::span::Span;

#[derive(Debug, Clone)]
pub enum Pattern {
    Ident(IdentPattern),
    Literal(LiteralPattern),
    Wildcard(Span),
    Ref(RefPattern),
    Struct(StructPattern),
    Tuple(TuplePattern),
    Enum(EnumPattern),
    Or(OrPattern),
}

impl Pattern {
    pub fn span(&self) -> Span {
        match self {
            Pattern::Ident(p) => p.span,
            Pattern::Literal(p) => p.span,
            Pattern::Wildcard(s) => *s,
            Pattern::Ref(p) => p.span,
            Pattern::Struct(p) => p.span,
            Pattern::Tuple(p) => p.span,
            Pattern::Enum(p) => p.span,
            Pattern::Or(p) => p.span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IdentPattern {
    pub name: String,
    pub is_mut: bool,
    pub is_ref: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LiteralPattern {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct RefPattern {
    pub is_mut: bool,
    pub inner: Box<Pattern>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructPattern {
    pub path: crate::expr::PathExpr,
    pub fields: Vec<(String, Option<Pattern>)>,
    pub rest: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TuplePattern {
    pub elements: Vec<Pattern>,
    pub rest: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumPattern {
    pub path: crate::expr::PathExpr,
    pub args: Option<Vec<Pattern>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct OrPattern {
    pub patterns: Vec<Pattern>,
    pub span: Span,
}

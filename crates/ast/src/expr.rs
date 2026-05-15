use crate::{
    pattern::Pattern,
    stmt::Block,
    ty::Ty,
};
use rt_common::span::Span;

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(LiteralExpr),
    Ident(IdentExpr),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Call(CallExpr),
    Index(IndexExpr),
    Field(FieldExpr),
    Path(PathExpr),
    Paren(ParenExpr),
    Block(Block),
    If(IfExpr),
    Loop(LoopExpr),
    While(WhileExpr),
    For(ForExpr),
    Match(MatchExpr),
    Return(ReturnExpr),
    Break(BreakExpr),
    Continue(Span),
    Assign(AssignExpr),
    Closure(ClosureExpr),
    Struct(StructExpr),
    Tuple(TupleExpr),
    Array(ArrayExpr),
    Range(RangeExpr),
    Underscore(Span),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(e) => e.span,
            Expr::Ident(e) => e.span,
            Expr::Binary(e) => e.span,
            Expr::Unary(e) => e.span,
            Expr::Call(e) => e.span,
            Expr::Index(e) => e.span,
            Expr::Field(e) => e.span,
            Expr::Path(e) => e.span,
            Expr::Paren(e) => e.span,
            Expr::Block(b) => b.span,
            Expr::If(e) => e.span,
            Expr::Loop(e) => e.span,
            Expr::While(e) => e.span,
            Expr::For(e) => e.span,
            Expr::Match(e) => e.span,
            Expr::Return(e) => e.span,
            Expr::Break(e) => e.span,
            Expr::Continue(s) => *s,
            Expr::Assign(e) => e.span,
            Expr::Closure(e) => e.span,
            Expr::Struct(e) => e.span,
            Expr::Tuple(e) => e.span,
            Expr::Array(e) => e.span,
            Expr::Range(e) => e.span,
            Expr::Underscore(s) => *s,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LiteralExpr {
    pub kind: LiteralKind,
    pub value: String,
    pub suffix: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralKind {
    Integer,
    Float,
    Char,
    Byte,
    String,
    ByteString,
    Bool,
}

#[derive(Debug, Clone)]
pub struct IdentExpr {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub op: BinOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Rem,
    And, Or, BitAnd, BitOr, BitXor,
    Shl, Shr,
    Eq, Ne, Lt, Gt, Le, Ge,
    AndAnd, OrOr,
    Assign, AddAssign, SubAssign, MulAssign, DivAssign, RemAssign,
    BitAndAssign, BitOrAssign, BitXorAssign, ShlAssign, ShrAssign,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg, Not, Deref, Ref, RefMut,
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub func: Box<Expr>,
    pub args: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IndexExpr {
    pub base: Box<Expr>,
    pub index: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FieldExpr {
    pub base: Box<Expr>,
    pub field: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct PathExpr {
    pub segments: Vec<PathSegment>,
    pub span: Span,
}

impl PathExpr {
    pub fn new_simple(name: String, span: Span) -> Self {
        PathExpr {
            segments: vec![PathSegment { name, args: None }],
            span,
        }
    }

    pub fn as_simple(&self) -> Option<&str> {
        if self.segments.len() == 1 && self.segments[0].args.is_none() {
            Some(&self.segments[0].name)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct PathSegment {
    pub name: String,
    pub args: Option<Vec<Ty>>,
}

#[derive(Debug, Clone)]
pub struct ParenExpr {
    pub expr: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IfExpr {
    pub condition: Box<Expr>,
    pub then_branch: Box<Block>,
    pub else_branch: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LoopExpr {
    pub body: Box<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct WhileExpr {
    pub condition: Box<Expr>,
    pub body: Box<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ForExpr {
    pub pattern: Pattern,
    pub iterable: Box<Expr>,
    pub body: Box<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchExpr {
    pub scrutinee: Box<Expr>,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    pub body: Expr,
}

#[derive(Debug, Clone)]
pub struct ReturnExpr {
    pub expr: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct BreakExpr {
    pub expr: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AssignExpr {
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ClosureExpr {
    pub params: Vec<(Pattern, Option<Ty>)>,
    pub body: Box<Expr>,
    pub is_move: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructExpr {
    pub path: PathExpr,
    pub fields: Vec<(String, Expr)>,
    pub base: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TupleExpr {
    pub elements: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ArrayExpr {
    pub elements: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct RangeExpr {
    pub lhs: Option<Box<Expr>>,
    pub rhs: Option<Box<Expr>>,
    pub inclusive: bool,
    pub span: Span,
}

use crate::expr::{Expr, PathExpr};
use rt_common::span::Span;

#[derive(Debug, Clone)]
pub enum Ty {
    Path(PathTy),
    Ref(RefTy),
    Tuple(TupleTy),
    Array(ArrayTy),
    Slice(SliceTy),
    Fn(FnTy),
    Unit(Span),
    Infer(Span),
}

impl Ty {
    pub fn span(&self) -> Span {
        match self {
            Ty::Path(t) => t.span,
            Ty::Ref(t) => t.span,
            Ty::Tuple(t) => t.span,
            Ty::Array(t) => t.span,
            Ty::Slice(t) => t.span,
            Ty::Fn(t) => t.span,
            Ty::Unit(s) | Ty::Infer(s) => *s,
        }
    }

    pub fn path(name: &str, span: Span) -> Self {
        Ty::Path(PathTy {
            path: PathExpr::new_simple(name.to_string(), span),
            span,
        })
    }

    pub fn path_named(name: String, span: Span) -> Self {
        Ty::Path(PathTy {
            path: PathExpr::new_simple(name, span),
            span,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PathTy {
    pub path: PathExpr,
    pub span: Span,
}

impl PathTy {
    pub fn as_simple(&self) -> Option<&str> {
        self.path.as_simple()
    }
}

#[derive(Debug, Clone)]
pub struct RefTy {
    pub is_mut: bool,
    pub inner: Box<Ty>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TupleTy {
    pub types: Vec<Ty>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ArrayTy {
    pub elem: Box<Ty>,
    pub len: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SliceTy {
    pub elem: Box<Ty>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FnTy {
    pub params: Vec<Ty>,
    pub ret: Box<Ty>,
    pub span: Span,
}

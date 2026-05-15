use crate::{
    expr::Expr,
    stmt::Block,
    ty::Ty,
};
use rt_common::span::Span;

#[derive(Debug, Clone)]
pub enum Item {
    Fn(FnItem),
    Struct(StructItem),
    Enum(EnumItem),
    Impl(ImplItem),
    Trait(TraitItem),
    Mod(ModItem),
    Use(UseItem),
    Const(ConstItem),
    Static(StaticItem),
    TypeAlias(TypeAliasItem),
}

impl Item {
    pub fn span(&self) -> Span {
        match self {
            Item::Fn(i) => i.span,
            Item::Struct(i) => i.span,
            Item::Enum(i) => i.span,
            Item::Impl(i) => i.span,
            Item::Trait(i) => i.span,
            Item::Mod(i) => i.span,
            Item::Use(i) => i.span,
            Item::Const(i) => i.span,
            Item::Static(i) => i.span,
            Item::TypeAlias(i) => i.span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FnItem {
    pub name: String,
    pub generics: Option<Generics>,
    pub params: Vec<FnParam>,
    pub ret_ty: Option<Ty>,
    pub body: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FnParam {
    pub pattern: crate::pattern::Pattern,
    pub ty: Ty,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructItem {
    pub name: String,
    pub generics: Option<Generics>,
    pub kind: StructKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum StructKind {
    Named(Vec<StructField>),
    Tuple(Vec<Ty>),
    Unit,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub ty: Ty,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumItem {
    pub name: String,
    pub generics: Option<Generics>,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Option<Vec<Ty>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ImplItem {
    pub generics: Option<Generics>,
    pub trait_path: Option<crate::expr::PathExpr>,
    pub ty: Ty,
    pub items: Vec<ImplItemKind>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ImplItemKind {
    Fn(FnItem),
    Const(ConstItem),
    TypeAlias(TypeAliasItem),
}

#[derive(Debug, Clone)]
pub struct TraitItem {
    pub name: String,
    pub generics: Option<Generics>,
    pub items: Vec<TraitItemKind>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TraitItemKind {
    Fn(FnItem),
    TypeAlias(TypeAliasItem),
    Const(ConstItem),
}

#[derive(Debug, Clone)]
pub struct ModItem {
    pub name: String,
    pub items: Option<Vec<Item>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct UseItem {
    pub path: crate::expr::PathExpr,
    pub alias: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ConstItem {
    pub name: String,
    pub ty: Option<Ty>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StaticItem {
    pub name: String,
    pub is_mut: bool,
    pub ty: Ty,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TypeAliasItem {
    pub name: String,
    pub generics: Option<Generics>,
    pub ty: Ty,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Generics {
    pub params: Vec<GenericParam>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum GenericParam {
    Type(String),
}

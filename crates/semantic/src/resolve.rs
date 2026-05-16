use std::sync::Arc;
use rt_ast::{
    expr::*,
    item::*,
    node::*,
    pattern::*,
    ty::Ty as AstTy,
};
use rt_common::{diagnostic::DiagnosticBag, span::Span};

use crate::{
    scope::{Scope, SymbolEntry, FnInfo, StructInfo, EnumInfo, TypeAliasInfo, VarInfo, TraitInfo, ModuleInfo},
    ty::{SemTy, StructTy, RefTy, FnTy},
};

pub struct NameResolver<'a> {
    scope: Scope,
    pub diagnostics: &'a mut DiagnosticBag,
    file: Arc<rt_common::span::SourceFile>,
}

impl<'a> NameResolver<'a> {
    pub fn new(diagnostics: &'a mut DiagnosticBag, file: Arc<rt_common::span::SourceFile>) -> Self {
        let mut scope = Scope::new();
        scope.insert("i32".into(), SymbolEntry::TypeAlias(TypeAliasInfo { ty: SemTy::I32 }));
        scope.insert("i64".into(), SymbolEntry::TypeAlias(TypeAliasInfo { ty: SemTy::I64 }));
        scope.insert("u32".into(), SymbolEntry::TypeAlias(TypeAliasInfo { ty: SemTy::U32 }));
        scope.insert("u64".into(), SymbolEntry::TypeAlias(TypeAliasInfo { ty: SemTy::U64 }));
        scope.insert("f32".into(), SymbolEntry::TypeAlias(TypeAliasInfo { ty: SemTy::F32 }));
        scope.insert("f64".into(), SymbolEntry::TypeAlias(TypeAliasInfo { ty: SemTy::F64 }));
        scope.insert("bool".into(), SymbolEntry::TypeAlias(TypeAliasInfo { ty: SemTy::Bool }));
        scope.insert("char".into(), SymbolEntry::TypeAlias(TypeAliasInfo { ty: SemTy::Char }));
        scope.insert("str".into(), SymbolEntry::TypeAlias(TypeAliasInfo { ty: SemTy::Str }));
        scope.insert("usize".into(), SymbolEntry::TypeAlias(TypeAliasInfo { ty: SemTy::U64 }));

        NameResolver { scope, diagnostics, file }
    }

    fn error(&mut self, msg: String, span: Span) {
        self.diagnostics.error(msg, span, self.file.clone());
    }

    pub fn resolve_program(&mut self, items: &[Item]) -> Scope {
        for item in items {
            self.declare_item(item);
        }

        let mut scope = Scope::new();
        scope.insert("i32".into(), SymbolEntry::TypeAlias(TypeAliasInfo { ty: SemTy::I32 }));
        scope.insert("i64".into(), SymbolEntry::TypeAlias(TypeAliasInfo { ty: SemTy::I64 }));
        scope.insert("u32".into(), SymbolEntry::TypeAlias(TypeAliasInfo { ty: SemTy::U32 }));
        scope.insert("u64".into(), SymbolEntry::TypeAlias(TypeAliasInfo { ty: SemTy::U64 }));
        scope.insert("f32".into(), SymbolEntry::TypeAlias(TypeAliasInfo { ty: SemTy::F32 }));
        scope.insert("f64".into(), SymbolEntry::TypeAlias(TypeAliasInfo { ty: SemTy::F64 }));
        scope.insert("bool".into(), SymbolEntry::TypeAlias(TypeAliasInfo { ty: SemTy::Bool }));
        scope.insert("char".into(), SymbolEntry::TypeAlias(TypeAliasInfo { ty: SemTy::Char }));
        scope.insert("str".into(), SymbolEntry::TypeAlias(TypeAliasInfo { ty: SemTy::Str }));
        scope.insert("usize".into(), SymbolEntry::TypeAlias(TypeAliasInfo { ty: SemTy::U64 }));

        scope.insert("Ok".into(), SymbolEntry::Fn(FnInfo { params: vec![SemTy::Infer], ret: Some(SemTy::Infer), generics: vec![] }));
        scope.insert("Err".into(), SymbolEntry::Fn(FnInfo { params: vec![SemTy::Infer], ret: Some(SemTy::Infer), generics: vec![] }));
        scope.insert("Some".into(), SymbolEntry::Fn(FnInfo { params: vec![SemTy::Infer], ret: Some(SemTy::Infer), generics: vec![] }));
        scope.insert("None".into(), SymbolEntry::Fn(FnInfo { params: vec![], ret: Some(SemTy::Infer), generics: vec![] }));
        scope.insert("println".into(), SymbolEntry::Fn(FnInfo { params: vec![SemTy::Infer], ret: Some(SemTy::Unit), generics: vec![] }));

        for item in items {
            self.resolve_item(item, &mut scope, &self.scope.clone());
        }

        scope
    }

    fn declare_item(&mut self, item: &Item) {
        match item {
            Item::Fn(fn_item) => {
                let fn_info = FnInfo {
                    params: vec![],
                    ret: None,
                    generics: fn_item.generics.as_ref()
                        .map(|g| g.params.iter().map(|p| match p {
                            GenericParam::Type(n) => n.clone()
                        }).collect())
                        .unwrap_or_default(),
                };
                self.scope.insert(fn_item.name.clone(), SymbolEntry::Fn(fn_info));
            }
            Item::Struct(s) => {
                let info = StructInfo {
                    fields: match &s.kind {
                        StructKind::Named(fields) => fields.iter()
                            .map(|f| (f.name.clone(), SemTy::Infer)).collect(),
                        StructKind::Tuple(types) => types.iter().enumerate()
                            .map(|(i, _)| (format!("_{}", i), SemTy::Infer)).collect(),
                        StructKind::Unit => vec![],
                    },
                    generics: s.generics.as_ref()
                        .map(|g| g.params.iter().map(|p| match p {
                            GenericParam::Type(n) => n.clone()
                        }).collect())
                        .unwrap_or_default(),
                };
                self.scope.insert(s.name.clone(), SymbolEntry::Struct(info));
            }
            Item::Enum(e) => {
                let info = EnumInfo {
                    variants: e.variants.iter()
                        .map(|v| (v.name.clone(), v.fields.clone().map(|tys| tys.iter().map(|_| SemTy::Infer).collect())))
                        .collect(),
                    generics: e.generics.as_ref()
                        .map(|g| g.params.iter().map(|p| match p {
                            GenericParam::Type(n) => n.clone()
                        }).collect())
                        .unwrap_or_default(),
                };
                self.scope.insert(e.name.clone(), SymbolEntry::Enum(info));
            }
            Item::Trait(t) => {
                let methods = t.items.iter().filter_map(|item| {
                    if let TraitItemKind::Fn(f) = item {
                        Some((f.name.clone(), vec![], None))
                    } else { None }
                }).collect();
                self.scope.insert(t.name.clone(), SymbolEntry::Trait(TraitInfo { methods }));
            }
            Item::Mod(m) => {
                if let Some(items) = &m.items {
                    let mut mod_scope = Scope::new();
                    for item in items {
                        self.declare_item(item);
                    }
                    self.scope.insert(m.name.clone(), SymbolEntry::Module(ModuleInfo { scope: Box::new(mod_scope) }));
                }
            }
            Item::TypeAlias(t) => {
                self.scope.insert(t.name.clone(), SymbolEntry::TypeAlias(TypeAliasInfo { ty: SemTy::Infer }));
            }
            _ => {}
        }
    }

    fn resolve_item(&mut self, item: &Item, scope: &mut Scope, global: &Scope) {
        match item {
            Item::Fn(fn_item) => {
                let ret_ty = fn_item.ret_ty.as_ref().map(|t| self.convert_ty(t));
                let fn_info = FnInfo {
                    params: fn_item.params.iter().map(|p| self.convert_ty(&p.ty)).collect(),
                    ret: ret_ty.clone(),
                    generics: vec![],
                };
                scope.insert(fn_item.name.clone(), SymbolEntry::Fn(fn_info.clone()));

                let mut fn_scope = Scope::child(scope);
                for param in &fn_item.params {
                    let name = match &param.pattern {
                        Pattern::Ident(p) => p.name.clone(),
                        _ => String::new(),
                    };
                    let ty = self.convert_ty(&param.ty);
                    fn_scope.insert(name, SymbolEntry::Var(VarInfo { ty: Some(ty), is_mut: false }));
                }
                fn_scope.insert(fn_item.name.clone(), SymbolEntry::Fn(fn_info));

                if let Some(body) = &fn_item.body {
                    self.resolve_block(body, &mut fn_scope, global);
                }
            }
            Item::Struct(s) => {
                let fields = match &s.kind {
                    StructKind::Named(fields) => fields.iter()
                        .map(|f| (f.name.clone(), self.convert_ty(&f.ty)))
                        .collect(),
                    _ => vec![],
                };
                scope.insert(s.name.clone(), SymbolEntry::Struct(StructInfo {
                    fields,
                    generics: vec![],
                }));
            }
            Item::Enum(e) => {
                let variants = e.variants.iter()
                    .map(|v| (v.name.clone(), v.fields.as_ref().map(|tys| tys.iter().map(|t| self.convert_ty(t)).collect())))
                    .collect();
                scope.insert(e.name.clone(), SymbolEntry::Enum(EnumInfo { variants, generics: vec![] }));
            }
            _ => {}
        }
    }

    fn convert_ty(&self, ast_ty: &AstTy) -> SemTy {
        match ast_ty {
            AstTy::Path(p) => {
                if let Some(name) = p.as_simple() {
                    SemTy::from_ident(name).unwrap_or_else(|| SemTy::Struct(StructTy {
                        name: name.to_string(),
                        fields: vec![],
                        generics: vec![],
                    }))
                } else {
                    SemTy::Infer
                }
            }
            AstTy::Ref(r) => {
                SemTy::Ref(Box::new(RefTy { inner: Box::new(self.convert_ty(&r.inner)), is_mut: r.is_mut }))
            }
            AstTy::Tuple(t) => {
                SemTy::Tuple(t.types.iter().map(|t| self.convert_ty(t)).collect())
            }
            AstTy::Array(a) => {
                SemTy::Array(Box::new(self.convert_ty(&a.elem)), 0)
            }
            AstTy::Slice(s) => {
                SemTy::Slice(Box::new(self.convert_ty(&s.elem)))
            }
            AstTy::Fn(f) => {
                SemTy::Fn(Box::new(FnTy {
                    params: f.params.iter().map(|t| self.convert_ty(t)).collect(),
                    ret: Box::new(self.convert_ty(&f.ret)),
                }))
            }
            AstTy::Unit(_) => SemTy::Unit,
            AstTy::Infer(_) => SemTy::Infer,
        }
    }

    fn resolve_block(&mut self, block: &rt_ast::stmt::Block, scope: &mut Scope, global: &Scope) {
        for stmt in &block.stmts {
            match stmt {
                Stmt::Let { pattern, ty, init, .. } => {
                    let sem_ty = ty.as_ref().map(|t| self.convert_ty(t));
                    if let Some(expr) = init {
                        self.resolve_expr(expr, scope, global);
                    }
                    let name = match pattern {
                        Pattern::Ident(p) => p.name.clone(),
                        _ => String::new(),
                    };
                    scope.insert(name, SymbolEntry::Var(VarInfo { ty: sem_ty, is_mut: false }));
                }
                Stmt::Expr(e) => {
                    self.resolve_expr(e, scope, global);
                }
                Stmt::Item(item) => {
                    self.resolve_item(item, scope, global);
                }
                _ => {}
            }
        }
        if let Some(expr) = &block.expr {
            self.resolve_expr(expr, scope, global);
        }
    }

    fn resolve_expr(&mut self, expr: &Expr, scope: &mut Scope, global: &Scope) {
        match expr {
            Expr::Ident(ident) => {
                if scope.lookup(&ident.name).is_none() && global.lookup(&ident.name).is_none() {
                    self.error(format!("cannot find value `{}` in this scope", ident.name), ident.span);
                }
            }
            Expr::Binary(b) => {
                self.resolve_expr(&b.lhs, scope, global);
                self.resolve_expr(&b.rhs, scope, global);
            }
            Expr::Unary(u) => {
                self.resolve_expr(&u.expr, scope, global);
            }
            Expr::Call(c) => {
                self.resolve_expr(&c.func, scope, global);
                for arg in &c.args {
                    self.resolve_expr(arg, scope, global);
                }
            }
            Expr::If(if_expr) => {
                self.resolve_expr(&if_expr.condition, scope, global);
                self.resolve_block(&if_expr.then_branch, scope, global);
                if let Some(else_expr) = &if_expr.else_branch {
                    self.resolve_expr(else_expr, scope, global);
                }
            }
            Expr::Block(b) => {
                let mut block_scope = Scope::child(scope);
                self.resolve_block(b, &mut block_scope, global);
            }
            Expr::Return(r) => {
                if let Some(e) = &r.expr {
                    self.resolve_expr(e, scope, global);
                }
            }
            Expr::Loop(loop_expr) => {
                self.resolve_block(&loop_expr.body, scope, global);
            }
            Expr::While(while_expr) => {
                self.resolve_expr(&while_expr.condition, scope, global);
                self.resolve_block(&while_expr.body, scope, global);
            }
            Expr::For(for_expr) => {
                let name = match &for_expr.pattern {
                    Pattern::Ident(p) => p.name.clone(),
                    _ => String::new(),
                };
                if !name.is_empty() {
                    scope.insert(name, SymbolEntry::Var(VarInfo { ty: Some(SemTy::I32), is_mut: false }));
                }
                self.resolve_expr(&for_expr.iterable, scope, global);
                self.resolve_block(&for_expr.body, scope, global);
            }
            Expr::Break(_) | Expr::Continue(_) => {}
            Expr::Match(match_expr) => {
                self.resolve_expr(&match_expr.scrutinee, scope, global);
                for arm in &match_expr.arms {
                    if let Some(guard) = &arm.guard {
                        self.resolve_expr(guard, scope, global);
                    }
                    let name = match &arm.pattern {
                        Pattern::Ident(p) => Some(p.name.clone()),
                        _ => None,
                    };
                    if let Some(n) = name {
                        scope.insert(n, SymbolEntry::Var(VarInfo { ty: None, is_mut: false }));
                    }
                    self.resolve_expr(&arm.body, scope, global);
                }
            }
            Expr::Tuple(tuple_expr) => {
                for elem in &tuple_expr.elements {
                    self.resolve_expr(elem, scope, global);
                }
            }
            Expr::Index(index_expr) => {
                self.resolve_expr(&index_expr.base, scope, global);
                self.resolve_expr(&index_expr.index, scope, global);
            }
            Expr::Try(try_expr) => {
                self.resolve_expr(&try_expr.expr, scope, global);
            }
            _ => {}
        }
    }
}

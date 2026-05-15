use rt_ast::expr::*;
use rt_ast::item::*;
use rt_ast::node::*;
use rt_common::diagnostic::DiagnosticBag;
use rt_common::span::{SourceFile, Span};
use std::sync::Arc;

use crate::scope::{Scope, SymbolEntry, VarInfo};
use crate::ty::SemTy;

pub struct TypeChecker<'a> {
    global_scope: &'a Scope,
    pub diagnostics: &'a mut DiagnosticBag,
    file: Arc<SourceFile>,
    local_scope: Scope,
}

impl<'a> TypeChecker<'a> {
    pub fn new(scope: &'a Scope, diagnostics: &'a mut DiagnosticBag, file: Arc<SourceFile>) -> Self {
        TypeChecker {
            global_scope: scope,
            diagnostics,
            file,
            local_scope: Scope::new(),
        }
    }

    fn error(&mut self, msg: String, span: Span) {
        self.diagnostics.error(msg, span, self.file.clone());
    }

    fn lookup(&self, name: &str) -> Option<&SymbolEntry> {
        self.local_scope.lookup(name).or_else(|| self.global_scope.lookup(name))
    }

    pub fn check_program(&mut self, items: &[Item]) {
        for item in items {
            self.check_item(item);
        }
    }

    fn check_item(&mut self, item: &Item) {
        match item {
            Item::Fn(fn_item) => {
                if let Some(body) = &fn_item.body {
                    let ret_ty = fn_item.ret_ty.as_ref()
                        .map(|t| self.ast_ty_to_sem(t));
                    let mut fn_scope = Scope::child(self.local_scope.clone());
                    std::mem::swap(&mut self.local_scope, &mut fn_scope);

                    for param in &fn_item.params {
                        let name = match &param.pattern {
                            rt_ast::pattern::Pattern::Ident(p) => p.name.clone(),
                            _ => String::new(),
                        };
                        let ty = self.ast_ty_to_sem(&param.ty);
                        self.local_scope.insert(name, SymbolEntry::Var(VarInfo { ty: Some(ty), is_mut: false }));
                    }

                    self.check_block(body, ret_ty);

                    std::mem::swap(&mut self.local_scope, &mut fn_scope);
                }
            }
            _ => {}
        }
    }

    fn ast_ty_to_sem(&self, ast_ty: &rt_ast::ty::Ty) -> SemTy {
        match ast_ty {
            rt_ast::ty::Ty::Path(p) => {
                p.as_simple().and_then(SemTy::from_ident).unwrap_or(SemTy::Infer)
            }
            rt_ast::ty::Ty::Ref(r) => {
                SemTy::Ref(Box::new(crate::ty::RefTy {
                    inner: Box::new(self.ast_ty_to_sem(&r.inner)),
                    is_mut: r.is_mut,
                }))
            }
            _ => SemTy::Infer,
        }
    }

    fn check_block(&mut self, block: &rt_ast::stmt::Block, expected_ret: Option<SemTy>) -> SemTy {
        let mut block_ty = SemTy::Unit;
        for stmt in &block.stmts {
            match stmt {
                Stmt::Let { pattern, ty, init, .. } => {
                    let sem_ty = ty.as_ref().map(|t| self.ast_ty_to_sem(t));
                    let init_ty = init.as_ref().map(|expr| self.infer_expr(expr, &expected_ret));
                    if let (Some(decl_ty), Some(inf_ty)) = (&sem_ty, &init_ty) {
                        if decl_ty != &SemTy::Infer && inf_ty != &SemTy::Infer && decl_ty != inf_ty {
                            self.error(
                                format!("mismatched types: expected {}, found {}", decl_ty.name(), inf_ty.name()),
                                init.as_ref().unwrap().span(),
                            );
                        }
                    }
                    let name = match pattern {
                        rt_ast::pattern::Pattern::Ident(p) => p.name.clone(),
                        _ => String::new(),
                    };
                    if !name.is_empty() {
                        if self.local_scope.lookup_local(&name).is_some() {
                            self.error(format!("duplicate declaration of `{}`", name), Span::DUMMY);
                        }
                        self.local_scope.insert(name, SymbolEntry::Var(VarInfo { ty: sem_ty, is_mut: false }));
                    }
                }
                Stmt::Expr(e) => {
                    let ty = self.infer_expr(e, &expected_ret);
                    if let Expr::Return(ret) = e {
                        if let Some(expected) = &expected_ret {
                            let actual = ret.expr.as_ref().map(|_| ty.clone()).unwrap_or(SemTy::Unit);
                            if expected != &SemTy::Infer && actual != SemTy::Infer && expected != &actual {
                                self.error(
                                    format!("mismatched return type: expected {}, found {}", expected.name(), actual.name()),
                                    ret.span,
                                );
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        if let Some(expr) = &block.expr {
            block_ty = self.infer_expr(expr, &expected_ret);
            if let Some(ret) = &expected_ret {
                if ret != &SemTy::Infer && block_ty != SemTy::Infer && ret != &block_ty {
                    self.error(
                        format!("mismatched types: expected {}, found {}", ret.name(), block_ty.name()),
                        expr.span(),
                    );
                }
            }
        }
        block_ty
    }

    fn infer_expr(&mut self, expr: &Expr, expected_ret: &Option<SemTy>) -> SemTy {
        match expr {
            Expr::Literal(lit) => {
                if let Some(suffix) = &lit.suffix {
                    SemTy::from_suffix(suffix).unwrap_or_else(|| match lit.kind {
                        LiteralKind::Integer => SemTy::default_int(),
                        LiteralKind::Float => SemTy::default_float(),
                        LiteralKind::Bool => SemTy::Bool,
                        LiteralKind::Char => SemTy::Char,
                        LiteralKind::String => SemTy::Str,
                        _ => SemTy::Infer,
                    })
                } else {
                    match lit.kind {
                        LiteralKind::Integer => SemTy::default_int(),
                        LiteralKind::Float => SemTy::default_float(),
                        LiteralKind::Bool => SemTy::Bool,
                        LiteralKind::Char => SemTy::Char,
                        LiteralKind::String => SemTy::Str,
                        _ => SemTy::Infer,
                    }
                }
            }
            Expr::Ident(ident) => {
                match self.lookup(&ident.name) {
                    Some(SymbolEntry::Var(v)) => v.ty.clone().unwrap_or(SemTy::Infer),
                    Some(SymbolEntry::Fn(_)) => SemTy::Infer,
                    _ => {
                        self.error(format!("cannot find value `{}`", ident.name), ident.span);
                        SemTy::Infer
                    }
                }
            }
            Expr::Binary(bin) => {
                let lhs_ty = self.infer_expr(&bin.lhs, &expected_ret);
                let rhs_ty = self.infer_expr(&bin.rhs, &expected_ret);
                match bin.op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => lhs_ty,
                    BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => SemTy::Bool,
                    BinOp::AndAnd | BinOp::OrOr => SemTy::Bool,
                    _ => lhs_ty,
                }
            }
            Expr::Unary(un) => {
                let ty = self.infer_expr(&un.expr, &expected_ret);
                match un.op {
                    UnaryOp::Not => SemTy::Bool,
                    _ => ty,
                }
            }
            Expr::Call(call) => {
                self.infer_expr(&call.func, &expected_ret);
            for arg in &call.args {
                    self.infer_expr(arg, &expected_ret);
                }
                SemTy::Infer
            }
            Expr::If(if_expr) => {
                let cond_ty = self.infer_expr(&if_expr.condition, &expected_ret);
                if cond_ty != SemTy::Bool && cond_ty != SemTy::Infer {
                    self.error("if condition must be bool".into(), if_expr.condition.span());
                }
                let then_ty = self.check_block(&if_expr.then_branch, None);
                let else_ty = if let Some(else_expr) = &if_expr.else_branch {
                    self.infer_expr(else_expr, &expected_ret)
                } else {
                    SemTy::Unit
                };
                if then_ty == else_ty { then_ty } else { SemTy::Unit }
            }
            Expr::Block(block) => {
                let mut block_scope = Scope::child(self.local_scope.clone());
                std::mem::swap(&mut self.local_scope, &mut block_scope);
                let ty = self.check_block(block, None);
                std::mem::swap(&mut self.local_scope, &mut block_scope);
                ty
            }
            Expr::Return(ret) => {
                if let Some(e) = &ret.expr {
                    self.infer_expr(e, &expected_ret)
                } else {
                    SemTy::Unit
                }
            }
            Expr::Loop(loop_expr) => {
                self.check_block(&loop_expr.body, None);
                SemTy::Unit
            }
            Expr::While(while_expr) => {
                self.infer_expr(&while_expr.condition, &expected_ret);
                self.check_block(&while_expr.body, None);
                SemTy::Unit
            }
            Expr::For(for_expr) => {
                self.infer_expr(&for_expr.iterable, &expected_ret);
                self.check_block(&for_expr.body, None);
                SemTy::Unit
            }
            _ => SemTy::Infer,
        }
    }
}

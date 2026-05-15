use rt_ast::expr::*;
use rt_ast::item::*;
use rt_ast::node::*;
use rt_common::diagnostic::DiagnosticBag;
use rt_common::span::{SourceFile, Span};
use std::sync::Arc;

use crate::scope::{Scope, SymbolEntry};
use crate::ty::SemTy;

pub struct TypeChecker<'a> {
    scope: &'a Scope,
    pub diagnostics: &'a mut DiagnosticBag,
    file: Arc<SourceFile>,
}

impl<'a> TypeChecker<'a> {
    pub fn new(scope: &'a Scope, diagnostics: &'a mut DiagnosticBag, file: Arc<SourceFile>) -> Self {
        TypeChecker { scope, diagnostics, file }
    }

    fn error(&mut self, msg: String, span: Span) {
        self.diagnostics.error(msg, span, self.file.clone());
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
                    self.check_block(body, fn_item, ret_ty);
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

    fn check_block(&mut self, block: &rt_ast::stmt::Block, current_fn: &FnItem, expected_ret: Option<SemTy>) -> SemTy {
        let mut block_ty = SemTy::Unit;
        for stmt in &block.stmts {
            match stmt {
                Stmt::Let { init, .. } => {
                    if let Some(expr) = init {
                        self.infer_expr(expr, current_fn);
                    }
                }
                Stmt::Expr(e) => {
                    self.infer_expr(e, current_fn);
                }
                _ => {}
            }
        }
        if let Some(expr) = &block.expr {
            block_ty = self.infer_expr(expr, current_fn);
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

    fn infer_expr(&mut self, expr: &Expr, current_fn: &FnItem) -> SemTy {
        match expr {
            Expr::Literal(lit) => match lit.kind {
                LiteralKind::Integer => SemTy::I32,
                LiteralKind::Float => SemTy::F64,
                LiteralKind::Bool => SemTy::Bool,
                LiteralKind::Char => SemTy::Char,
                LiteralKind::String => SemTy::Str,
                _ => SemTy::Infer,
            },
            Expr::Ident(ident) => {
                match self.scope.lookup(&ident.name) {
                    Some(SymbolEntry::Var(v)) => v.ty.clone().unwrap_or(SemTy::Infer),
                    Some(SymbolEntry::Fn(_)) => SemTy::Fn(Box::new(crate::ty::FnTy {
                        params: vec![],
                        ret: Box::new(SemTy::Infer),
                    })),
                    _ => {
                        self.error(format!("cannot find value `{}`", ident.name), ident.span);
                        SemTy::Infer
                    }
                }
            }
            Expr::Binary(bin) => {
                let lhs_ty = self.infer_expr(&bin.lhs, current_fn);
                let rhs_ty = self.infer_expr(&bin.rhs, current_fn);
                match bin.op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => {
                        if lhs_ty != rhs_ty && lhs_ty != SemTy::Infer && rhs_ty != SemTy::Infer {
                            self.error("type mismatch in binary operation".into(), bin.span);
                        }
                        lhs_ty
                    }
                    BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => SemTy::Bool,
                    BinOp::AndAnd | BinOp::OrOr => SemTy::Bool,
                    _ => lhs_ty,
                }
            }
            Expr::Unary(un) => {
                let ty = self.infer_expr(&un.expr, current_fn);
                match un.op {
                    UnaryOp::Neg => ty,
                    UnaryOp::Not => SemTy::Bool,
                    UnaryOp::Ref => SemTy::Ref(Box::new(crate::ty::RefTy { inner: Box::new(ty), is_mut: false })),
                    UnaryOp::RefMut => SemTy::Ref(Box::new(crate::ty::RefTy { inner: Box::new(ty), is_mut: true })),
                    UnaryOp::Deref => ty,
                }
            }
            Expr::Call(call) => {
                self.infer_expr(&call.func, current_fn);
                for arg in &call.args {
                    self.infer_expr(arg, current_fn);
                }
                SemTy::Infer
            }
            Expr::If(if_expr) => {
                let cond_ty = self.infer_expr(&if_expr.condition, current_fn);
                if cond_ty != SemTy::Bool && cond_ty != SemTy::Infer {
                    self.error("if condition must be bool".into(), if_expr.condition.span());
                }
                let then_ty = self.check_block(&if_expr.then_branch, current_fn, None);
                if let Some(else_expr) = &if_expr.else_branch {
                    let else_ty = self.infer_expr(else_expr, current_fn);
                    if then_ty != else_ty && then_ty != SemTy::Infer && else_ty != SemTy::Infer {
                        self.error("if/else branches have incompatible types".into(), if_expr.span);
                    }
                }
                then_ty
            }
            Expr::Block(block) => {
                self.check_block(block, current_fn, None)
            }
            Expr::Return(ret) => {
                let ret_ty = if let Some(e) = &ret.expr {
                    self.infer_expr(e, current_fn)
                } else {
                    SemTy::Unit
                };
                ret_ty
            }
            Expr::Loop(loop_expr) => {
                self.check_block(&loop_expr.body, current_fn, None);
                SemTy::Unit
            }
            Expr::While(while_expr) => {
                let cond_ty = self.infer_expr(&while_expr.condition, current_fn);
                if cond_ty != SemTy::Bool && cond_ty != SemTy::Infer {
                    self.error("while condition must be bool".into(), while_expr.condition.span());
                }
                self.check_block(&while_expr.body, current_fn, None);
                SemTy::Unit
            }
            Expr::For(for_expr) => {
                self.infer_expr(&for_expr.iterable, current_fn);
                self.check_block(&for_expr.body, current_fn, None);
                SemTy::Unit
            }
            _ => SemTy::Infer,
        }
    }
}

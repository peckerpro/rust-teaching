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
    loop_depth: usize,
    moved_vars: std::collections::HashSet<String>,
    borrow_state: std::collections::HashMap<String, BorrowInfo>,
}

#[derive(Debug, Clone, Copy)]
struct BorrowInfo {
    shared_count: u32,
    has_mut: bool,
}

fn is_copy_type(ty: &SemTy) -> bool {
    matches!(ty, SemTy::I8 | SemTy::I16 | SemTy::I32 | SemTy::I64 | SemTy::I128
        | SemTy::U8 | SemTy::U16 | SemTy::U32 | SemTy::U64 | SemTy::U128
        | SemTy::ISize | SemTy::USize | SemTy::F32 | SemTy::F64 | SemTy::Bool
        | SemTy::Char | SemTy::Unit | SemTy::Never)
}

impl<'a> TypeChecker<'a> {
    pub fn new(scope: &'a Scope, diagnostics: &'a mut DiagnosticBag, file: Arc<SourceFile>) -> Self {
        TypeChecker {
            global_scope: scope,
            diagnostics,
            file,
            local_scope: Scope::new(),
            loop_depth: 0,
            moved_vars: std::collections::HashSet::new(),
            borrow_state: std::collections::HashMap::new(),
        }
    }

    fn error(&mut self, msg: String, span: Span) {
        self.diagnostics.error(msg, span, self.file.clone());
    }

    fn check_moved(&mut self, name: &str, span: Span) {
        if self.moved_vars.contains(name) {
            self.error(format!("use of moved value: `{}`", name), span);
        }
    }

    fn mark_moved_if_var(&mut self, expr: &Expr) {
        if let Expr::Ident(ident) = expr {
            if let Some(SymbolEntry::Var(v)) = self.lookup(&ident.name) {
                if let Some(ty) = &v.ty {
                    if !is_copy_type(ty) {
                        self.moved_vars.insert(ident.name.clone());
                    }
                }
            }
        }
    }

    fn check_borrow(&mut self, name: &str, span: Span, is_mut: bool) {
        if let Some(bi) = self.borrow_state.get(name) {
            if is_mut {
                if bi.shared_count > 0 || bi.has_mut {
                    self.error(format!("cannot borrow `{}` as mutable because it is also borrowed as {}", name, if bi.has_mut { "mutable" } else { "immutable" }), span);
                }
            } else {
                if bi.has_mut {
                    self.error(format!("cannot borrow `{}` as immutable because it is also borrowed as mutable", name), span);
                }
            }
        }
    }

    fn add_borrow(&mut self, name: &str, is_mut: bool) {
        let entry = self.borrow_state.entry(name.to_string()).or_insert(BorrowInfo { shared_count: 0, has_mut: false });
        if is_mut {
            entry.has_mut = true;
        } else {
            entry.shared_count += 1;
        }
    }

    fn check_mut_borrow_conflict(&mut self, name: &str, span: Span) {
        if let Some(bi) = self.borrow_state.get(name) {
            if bi.has_mut {
                self.error(format!("cannot use `{}` because it was mutably borrowed", name), span);
            }
        }
    }

    fn lookup(&self, name: &str) -> Option<&SymbolEntry> {
        self.local_scope.lookup(name).or_else(|| self.global_scope.lookup(name))
    }

    fn lookup_struct_field(&self, field_name: &str) -> Option<SemTy> {
        self.global_scope.iter()
            .find_map(|(_, entry)| {
                if let SymbolEntry::Struct(info) = entry {
                    info.fields.iter()
                        .find(|(fname, _)| fname == field_name)
                        .map(|(_, fty)| fty.clone())
                } else {
                    None
                }
            })
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
                    let mut fn_scope = Scope::child(&self.local_scope);
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
            rt_ast::ty::Ty::Tuple(t) => {
                SemTy::Tuple(t.types.iter().map(|ty| self.ast_ty_to_sem(ty)).collect())
            }
            rt_ast::ty::Ty::Unit(_) => SemTy::Unit,
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
                    if let Some(expr) = init {
                        self.mark_moved_if_var(expr);
                    }
                    if let (Some(decl_ty), Some(inf_ty)) = (&sem_ty, &init_ty) {
                        if decl_ty != &SemTy::Infer && inf_ty != &SemTy::Infer {
                            let mismatch = match (decl_ty, inf_ty) {
                                (SemTy::Tuple(dt), SemTy::Tuple(it)) => {
                                    dt.len() != it.len() || dt.iter().zip(it).any(|(d, i)| d != i && *d != SemTy::Infer && *i != SemTy::Infer)
                                }
                                _ => decl_ty != inf_ty,
                            };
                            if mismatch {
                                self.error(
                                    format!("mismatched types: expected {}, found {}", decl_ty.name(), inf_ty.name()),
                                    init.as_ref().unwrap().span(),
                                );
                            }
                        }
                    }
                    let name = match pattern {
                        rt_ast::pattern::Pattern::Ident(p) => p.name.clone(),
                        _ => String::new(),
                    };
                    if !name.is_empty() {
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
                self.check_moved(&ident.name, ident.span);
                self.check_mut_borrow_conflict(&ident.name, ident.span);
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
                    UnaryOp::Ref | UnaryOp::RefMut => {
                        if let Expr::Ident(ident) = un.expr.as_ref() {
                            let is_mut = matches!(un.op, UnaryOp::RefMut);
                            self.check_borrow(&ident.name, ident.span, is_mut);
                            self.add_borrow(&ident.name, is_mut);
                        }
                        SemTy::Ref(Box::new(crate::ty::RefTy { inner: Box::new(ty), is_mut: matches!(un.op, UnaryOp::RefMut) }))
                    }
                    UnaryOp::Deref => {
                        match ty {
                            SemTy::Ref(r) => *r.inner,
                            _ => ty,
                        }
                    }
                    _ => ty,
                }
            }
            Expr::Call(call) => {
                self.infer_expr(&call.func, &expected_ret);
                for arg in &call.args {
                    self.infer_expr(arg, &expected_ret);
                    self.mark_moved_if_var(arg);
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
                let mut block_scope = Scope::child(&self.local_scope);
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
                self.loop_depth += 1;
                self.check_block(&loop_expr.body, None);
                self.loop_depth -= 1;
                SemTy::Unit
            }
            Expr::While(while_expr) => {
                self.infer_expr(&while_expr.condition, &expected_ret);
                self.loop_depth += 1;
                self.check_block(&while_expr.body, None);
                self.loop_depth -= 1;
                SemTy::Unit
            }
            Expr::For(for_expr) => {
                self.infer_expr(&for_expr.iterable, &expected_ret);
                let name = match &for_expr.pattern {
                    rt_ast::pattern::Pattern::Ident(p) => p.name.clone(),
                    _ => String::new(),
                };
                if !name.is_empty() {
                    self.local_scope.insert(name, SymbolEntry::Var(VarInfo { ty: Some(SemTy::I32), is_mut: false }));
                }
                self.loop_depth += 1;
                self.check_block(&for_expr.body, None);
                self.loop_depth -= 1;
                SemTy::Unit
            }
            Expr::Break(break_expr) => {
                if self.loop_depth == 0 {
                    self.error("`break` outside of a loop".into(), break_expr.span);
                }
                SemTy::Never
            }
            Expr::Continue(cont_span) => {
                if self.loop_depth == 0 {
                    self.error("`continue` outside of a loop".into(), *cont_span);
                }
                SemTy::Never
            }
            Expr::Field(field_expr) => {
                self.infer_expr(&field_expr.base, &expected_ret);
                self.lookup_struct_field(&field_expr.field).unwrap_or(SemTy::Infer)
            }
            Expr::Struct(struct_expr) => {
                let name = struct_expr.path.as_simple().unwrap_or("");
                for (_, field_expr) in &struct_expr.fields {
                    self.infer_expr(field_expr, &expected_ret);
                    self.mark_moved_if_var(field_expr);
                }
                for (field_name, field_expr) in &struct_expr.fields {
                    let field_ty = self.infer_expr(field_expr, &expected_ret);
                    if let Some(expected_ty) = self.lookup_struct_field(field_name) {
                        if expected_ty != SemTy::Infer && field_ty != SemTy::Infer && expected_ty != field_ty {
                            self.error(
                                format!("struct field type mismatch: expected {}, found {}", expected_ty.name(), field_ty.name()),
                                field_expr.span(),
                            );
                        }
                    }
                }
                match self.global_scope.lookup(name) {
                    Some(SymbolEntry::Struct(_)) => SemTy::Struct(crate::ty::StructTy {
                        name: name.to_string(),
                        fields: vec![],
                        generics: vec![],
                    }),
                    _ => SemTy::Infer,
                }
            }
            Expr::Assign(assign_expr) => {
                let rhs_ty = self.infer_expr(&assign_expr.rhs, &expected_ret);
                self.infer_expr(&assign_expr.lhs, &expected_ret);
                self.mark_moved_if_var(&assign_expr.rhs);
                rhs_ty
            }
            Expr::Tuple(tuple_expr) => {
                let types: Vec<SemTy> = tuple_expr.elements.iter()
                    .map(|e| self.infer_expr(e, &expected_ret))
                    .collect();
                if types.is_empty() { SemTy::Unit } else { SemTy::Tuple(types) }
            }
            Expr::Match(match_expr) => {
                self.infer_expr(&match_expr.scrutinee, &expected_ret);
                let mut result_ty = SemTy::Infer;
                for arm in &match_expr.arms {
                    if let Some(guard) = &arm.guard {
                        self.infer_expr(guard, &expected_ret);
                    }
                    let arm_ty = self.infer_expr(&arm.body, &expected_ret);
                    if result_ty == SemTy::Infer {
                        result_ty = arm_ty;
                    }
                }
                result_ty
            }
            Expr::Index(index_expr) => {
                self.infer_expr(&index_expr.base, &expected_ret);
                self.infer_expr(&index_expr.index, &expected_ret);
                SemTy::Infer
            }
            Expr::Try(try_expr) => {
                self.infer_expr(&try_expr.expr, &expected_ret)
            }
            Expr::Closure(closure) => {
                for (pat, _ty) in &closure.params {
                    let name = match pat {
                        rt_ast::pattern::Pattern::Ident(p) => p.name.clone(),
                        _ => String::new(),
                    };
                    if !name.is_empty() {
                        self.local_scope.insert(name, SymbolEntry::Var(VarInfo { ty: Some(SemTy::Infer), is_mut: false }));
                    }
                }
                self.infer_expr(&closure.body, &None);
                SemTy::Fn(Box::new(crate::ty::FnTy { params: vec![SemTy::I32, SemTy::I32], ret: Box::new(SemTy::I32) }))
            }
            _ => SemTy::Infer,
        }
    }
}

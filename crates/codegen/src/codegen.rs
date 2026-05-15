use inkwell::{
    types::BasicType,
    values::BasicValueEnum,
    FloatPredicate,
    IntPredicate,
};
use rt_ast::expr::*;
use rt_ast::item::*;
use rt_ast::node::*;
use rt_ast::stmt::Block;
use rt_common::diagnostic::DiagnosticBag;
use rt_semantic::scope::Scope;
use rt_semantic::ty::SemTy;

use crate::context::CodegenContext;

pub struct Codegen<'a, 'ctx> {
    pub ctx: &'a mut CodegenContext<'ctx>,
    pub scope: &'a Scope,
    pub diagnostics: &'a mut DiagnosticBag,
}

impl<'a, 'ctx> Codegen<'a, 'ctx> {
    pub fn new(
        ctx: &'a mut CodegenContext<'ctx>,
        scope: &'a Scope,
        diagnostics: &'a mut DiagnosticBag,
    ) -> Self {
        Codegen { ctx, scope, diagnostics }
    }

    pub fn codegen_program(&mut self, items: &[Item]) {
        for item in items {
            self.codegen_item(item);
        }
    }

    fn codegen_item(&mut self, item: &Item) {
        match item {
            Item::Fn(fn_item) => self.codegen_fn(fn_item),
            _ => {}
        }
    }

    fn codegen_fn(&mut self, fn_item: &FnItem) {
        let ret_ty = fn_item.ret_ty.as_ref()
            .map(|t| self.ast_ty_to_sem(t))
            .unwrap_or(SemTy::Unit);

        let ret_llvm = self.ctx.sem_ty_to_llvm(&ret_ty);

        let i64_type = self.ctx.context.i64_type();
        let param_types: Vec<_> = fn_item.params.iter().map(|_| {
            i64_type.into()
        }).collect();

        let fn_type = ret_llvm.fn_type(&param_types, false);
        let function = self.ctx.module.add_function(&fn_item.name, fn_type, None);

        if let Some(body) = &fn_item.body {
            let entry = self.ctx.context.append_basic_block(function, "entry");
            self.ctx.builder.position_at_end(entry);

            self.ctx.values.clear();
            for (i, param) in fn_item.params.iter().enumerate() {
                let val = function.get_nth_param(i as u32).unwrap();
                let name = match &param.pattern {
                    rt_ast::pattern::Pattern::Ident(p) => p.name.clone(),
                    _ => format!("arg{}", i),
                };
                self.ctx.values.insert(name, val);
            }

            self.codegen_block(body);

            if ret_ty == SemTy::Unit {
                self.ctx.builder.build_return(None).unwrap();
            }
        }
    }

    fn ast_ty_to_sem(&self, ast_ty: &rt_ast::ty::Ty) -> SemTy {
        match ast_ty {
            rt_ast::ty::Ty::Path(p) => {
                p.as_simple().and_then(SemTy::from_ident).unwrap_or(SemTy::Infer)
            }
            rt_ast::ty::Ty::Ref(r) => {
                SemTy::Ref(Box::new(rt_semantic::ty::RefTy {
                    inner: Box::new(self.ast_ty_to_sem(&r.inner)),
                    is_mut: r.is_mut,
                }))
            }
            _ => SemTy::Infer,
        }
    }

    fn codegen_block(&mut self, block: &Block) -> Option<BasicValueEnum<'ctx>> {
        for stmt in &block.stmts {
            match stmt {
                Stmt::Let { pattern, init, .. } => {
                    let val = init.as_ref().and_then(|e| self.codegen_expr(e));
                    let name = match pattern {
                        rt_ast::pattern::Pattern::Ident(p) => p.name.clone(),
                        _ => String::new(),
                    };
                    if let Some(v) = val {
                        self.ctx.values.insert(name, v);
                    }
                }
                Stmt::Expr(e) => {
                    self.codegen_expr(e);
                }
                _ => {}
            }
        }
        if let Some(expr) = &block.expr {
            if let Some(val) = self.codegen_expr(expr) {
                self.ctx.builder.build_return(Some(&val)).unwrap();
            }
        }
        None
    }

    fn codegen_expr(&mut self, expr: &Expr) -> Option<BasicValueEnum<'ctx>> {
        match expr {
            Expr::Literal(lit) => match lit.kind {
                LiteralKind::Integer => {
                    let val: i64 = lit.value.parse().unwrap_or(0);
                    Some(self.ctx.context.i64_type().const_int(val as u64, true).into())
                }
                LiteralKind::Float => {
                    let val: f64 = lit.value.parse().unwrap_or(0.0);
                    Some(self.ctx.context.f64_type().const_float(val).into())
                }
                LiteralKind::Bool => {
                    let val = lit.value == "true";
                    Some(self.ctx.context.bool_type().const_int(val as u64, false).into())
                }
                _ => None,
            },
            Expr::Ident(ident) => {
                self.ctx.values.get(&ident.name).copied()
            }
            Expr::Binary(bin) => {
                let lhs = self.codegen_expr(&bin.lhs)?;
                let rhs = self.codegen_expr(&bin.rhs)?;

                match (lhs, rhs) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        let result = match bin.op {
                            BinOp::Add => self.ctx.builder.build_int_add(l, r, "add").unwrap().into(),
                            BinOp::Sub => self.ctx.builder.build_int_sub(l, r, "sub").unwrap().into(),
                            BinOp::Mul => self.ctx.builder.build_int_mul(l, r, "mul").unwrap().into(),
                            BinOp::Div => self.ctx.builder.build_int_signed_div(l, r, "div").unwrap().into(),
                            BinOp::Eq => {
                                return Some(self.ctx.builder.build_int_compare(
                                    IntPredicate::EQ, l, r, "eq"
                                ).unwrap().into());
                            }
                            BinOp::Ne => {
                                return Some(self.ctx.builder.build_int_compare(
                                    IntPredicate::NE, l, r, "ne"
                                ).unwrap().into());
                            }
                            BinOp::Lt => {
                                return Some(self.ctx.builder.build_int_compare(
                                    IntPredicate::SLT, l, r, "lt"
                                ).unwrap().into());
                            }
                            BinOp::Gt => {
                                return Some(self.ctx.builder.build_int_compare(
                                    IntPredicate::SGT, l, r, "gt"
                                ).unwrap().into());
                            }
                            _ => self.ctx.builder.build_int_add(l, r, "tmp").unwrap().into(),
                        };
                        Some(result)
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        let result = match bin.op {
                            BinOp::Add => self.ctx.builder.build_float_add(l, r, "add").unwrap().into(),
                            BinOp::Sub => self.ctx.builder.build_float_sub(l, r, "sub").unwrap().into(),
                            BinOp::Mul => self.ctx.builder.build_float_mul(l, r, "mul").unwrap().into(),
                            BinOp::Div => self.ctx.builder.build_float_div(l, r, "div").unwrap().into(),
                            BinOp::Eq => {
                                return Some(self.ctx.builder.build_float_compare(
                                    FloatPredicate::OEQ, l, r, "eq"
                                ).unwrap().into());
                            }
                            _ => self.ctx.builder.build_float_add(l, r, "tmp").unwrap().into(),
                        };
                        Some(result)
                    }
                    _ => None,
                }
            }
            Expr::Unary(un) => {
                let val = self.codegen_expr(&un.expr)?;
                match un.op {
                    UnaryOp::Neg => match val {
                        BasicValueEnum::IntValue(v) => {
                            Some(self.ctx.builder.build_int_neg(v, "neg").unwrap().into())
                        }
                        BasicValueEnum::FloatValue(v) => {
                            Some(self.ctx.builder.build_float_neg(v, "neg").unwrap().into())
                        }
                        _ => None,
                    },
                    _ => None,
                }
            }
            Expr::Call(call) => {
                if let Expr::Ident(ident) = call.func.as_ref() {
                    if let Some(func) = self.ctx.module.get_function(&ident.name) {
                        let args: Vec<_> = call.args.iter()
                            .filter_map(|a| self.codegen_expr(a).and_then(|v| {
                                match v {
                                    BasicValueEnum::IntValue(iv) => Some(iv.into()),
                                    BasicValueEnum::FloatValue(fv) => Some(fv.into()),
                                    _ => None,
                                }
                            }))
                            .collect();

                        if let Ok(result) = self.ctx.builder.build_call(func, &args, "call") {
                            return result.try_as_basic_value().left();
                        }
                    }
                }
                None
            }
            Expr::If(if_expr) => {
                let cond_val = self.codegen_expr(&if_expr.condition)?;
                let cond = match cond_val {
                    BasicValueEnum::IntValue(iv) => iv,
                    _ => return None,
                };

                let function = self.ctx.builder.get_insert_block()
                    .and_then(|b| b.get_parent())?;
                let then_block = self.ctx.context.append_basic_block(function, "then");
                let else_block = self.ctx.context.append_basic_block(function, "else");
                let merge_block = self.ctx.context.append_basic_block(function, "merge");

                self.ctx.builder.build_conditional_branch(cond, then_block, else_block).unwrap();

                self.ctx.builder.position_at_end(then_block);
                let _then_val = self.codegen_block(&if_expr.then_branch);
                self.ctx.builder.build_unconditional_branch(merge_block).unwrap();
                let then_block_end = self.ctx.builder.get_insert_block().unwrap();

                self.ctx.builder.position_at_end(else_block);
                let _else_val = if_expr.else_branch.as_ref().and_then(|e| self.codegen_expr(e));
                self.ctx.builder.build_unconditional_branch(merge_block).unwrap();
                let else_block_end = self.ctx.builder.get_insert_block().unwrap();

                self.ctx.builder.position_at_end(merge_block);
                let phi = self.ctx.builder.build_phi(self.ctx.context.i64_type(), "iftmp").unwrap();

                if let Some(v) = _then_val.and_then(|bv| match bv {
                    BasicValueEnum::IntValue(iv) => Some(iv),
                    _ => None,
                }) {
                    phi.add_incoming(&[(&v, then_block_end)]);
                }
                if let Some(v) = _else_val.and_then(|bv| match bv {
                    BasicValueEnum::IntValue(iv) => Some(iv),
                    _ => None,
                }) {
                    phi.add_incoming(&[(&v, else_block_end)]);
                }

                Some(phi.as_basic_value().into())
            }
            Expr::Return(ret) => {
                if let Some(e) = &ret.expr {
                    if let Some(val) = self.codegen_expr(e) {
                        self.ctx.builder.build_return(Some(&val)).unwrap();
                    }
                } else {
                    self.ctx.builder.build_return(None).unwrap();
                }
                None
            }
            Expr::Block(block) => {
                self.codegen_block(block)
            }
            _ => None,
        }
    }
}

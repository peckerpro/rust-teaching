use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::values::{FunctionValue, BasicValueEnum};
use inkwell::types::{BasicType, BasicTypeEnum};
use std::collections::HashMap;
use rt_semantic::ty::SemTy;

pub struct CodegenContext<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub values: HashMap<String, BasicValueEnum<'ctx>>,
}

impl<'ctx> CodegenContext<'ctx> {
    pub fn new(context: &'ctx Context, name: &str) -> Self {
        let module = context.create_module(name);
        let builder = context.create_builder();
        CodegenContext {
            context,
            module,
            builder,
            values: HashMap::new(),
        }
    }

    pub fn sem_ty_to_llvm(&self, ty: &SemTy) -> BasicTypeEnum<'ctx> {
        match ty {
            SemTy::I32 => self.context.i32_type().into(),
            SemTy::I64 => self.context.i64_type().into(),
            SemTy::U32 => self.context.i32_type().into(),
            SemTy::U64 => self.context.i64_type().into(),
            SemTy::F32 => self.context.f32_type().into(),
            SemTy::F64 => self.context.f64_type().into(),
            SemTy::Bool => self.context.bool_type().into(),
            SemTy::Char => self.context.i8_type().into(),
            SemTy::Unit => self.context.struct_type(&[], false).into(),
            SemTy::Ref(inner) => {
                let inner_ty = self.sem_ty_to_llvm(&inner.inner);
                inner_ty.ptr_type(inkwell::AddressSpace::default()).into()
            }
            _ => self.context.i64_type().into(),
        }
    }
}

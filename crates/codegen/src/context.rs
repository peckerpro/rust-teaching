use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::values::BasicValueEnum;
use inkwell::types::{BasicType, BasicTypeEnum, StructType};
use inkwell::basic_block::BasicBlock;
use std::collections::HashMap;
use rt_semantic::ty::SemTy;
use rt_ast::item::FnItem as AstFnItem;

pub struct CodegenContext<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub values: HashMap<String, BasicValueEnum<'ctx>>,
    pub loop_stack: Vec<(BasicBlock<'ctx>, BasicBlock<'ctx>)>,
    pub struct_types: HashMap<String, (StructType<'ctx>, Vec<String>)>,
    pub enum_types: HashMap<String, (StructType<'ctx>, Vec<(String, Option<Vec<BasicTypeEnum<'ctx>>>)>)>,
    pub generic_templates: HashMap<String, AstFnItem>,
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
            loop_stack: Vec::new(),
            struct_types: HashMap::new(),
            enum_types: HashMap::new(),
            generic_templates: HashMap::new(),
        }
    }

    pub fn declare_struct(&mut self, name: &str, fields: &[(String, SemTy)]) {
        let llvm_fields: Vec<BasicTypeEnum<'ctx>> = fields
            .iter()
            .map(|(_, ty)| self.sem_ty_to_llvm(ty))
            .collect();
        let field_names: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();

        if let Some(struct_type) = self.context.get_struct_type(name) {
            struct_type.set_body(&llvm_fields, false);
            self.struct_types.insert(name.to_string(), (struct_type, field_names));
        } else {
            let struct_type = self.context.opaque_struct_type(name);
            struct_type.set_body(&llvm_fields, false);
            self.struct_types.insert(name.to_string(), (struct_type, field_names));
        }
    }

    pub fn sem_ty_to_llvm(&self, ty: &SemTy) -> BasicTypeEnum<'ctx> {
        match ty {
            SemTy::I8 | SemTy::U8 => self.context.i8_type().into(),
            SemTy::I16 | SemTy::U16 => self.context.i16_type().into(),
            SemTy::I32 | SemTy::U32 => self.context.i32_type().into(),
            SemTy::I64 | SemTy::U64 => self.context.i64_type().into(),
            SemTy::I128 | SemTy::U128 => self.context.i128_type().into(),
            SemTy::ISize | SemTy::USize => self.context.i64_type().into(),
            SemTy::F32 => self.context.f32_type().into(),
            SemTy::F64 => self.context.f64_type().into(),
            SemTy::Bool => self.context.bool_type().into(),
            SemTy::Char => self.context.i8_type().into(),
            SemTy::Str | SemTy::String => self.context.i8_type().ptr_type(inkwell::AddressSpace::default()).into(),
            SemTy::Unit => self.context.struct_type(&[], false).into(),
            SemTy::Ref(inner) => {
                let inner_ty = self.sem_ty_to_llvm(&inner.inner);
                inner_ty.ptr_type(inkwell::AddressSpace::default()).into()
            }
            _ => self.context.i64_type().into(),
        }
    }
}

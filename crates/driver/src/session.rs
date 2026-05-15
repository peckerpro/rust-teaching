use inkwell::context::Context;
use rt_ast::item::Item;
use rt_common::{
    diagnostic::DiagnosticBag,
    span::SourceFile,
};
use rt_lexer::Lexer;
use rt_parser::Parser;
use rt_semantic::{resolve::NameResolver, typeck::TypeChecker};
use rt_codegen::{codegen::Codegen, context::CodegenContext};
use std::sync::Arc;

pub struct Session {
    pub diagnostics: DiagnosticBag,
    pub file: Arc<SourceFile>,
    pub output_ir: bool,
}

impl Session {
    pub fn new(source: String, filename: String, output_ir: bool) -> Self {
        let file = Arc::new(SourceFile::new(filename, source));
        Session {
            diagnostics: DiagnosticBag::new(),
            file,
            output_ir,
        }
    }

    pub fn compile(&mut self) -> Result<(), ()> {
        let src = self.file.src.clone();
        let file = self.file.clone();

        let lexer = Lexer::new(&src, file.clone());
        let mut parser = Parser::new(lexer, file.clone());

        let items: Vec<Item> = parser.parse_program();

        if parser.diagnostics.has_errors() {
            for d in parser.diagnostics.diagnostics() {
                self.diagnostics.error(d.message.clone(), d.span, file.clone());
            }
            return Err(());
        }

        let mut resolver = NameResolver::new(&mut self.diagnostics, file.clone());
        let scope = resolver.resolve_program(&items);

        if self.diagnostics.has_errors() {
            return Err(());
        }

        let mut typeck = TypeChecker::new(&scope, &mut self.diagnostics, file.clone());
        typeck.check_program(&items);

        if self.diagnostics.has_errors() {
            return Err(());
        }

        let context = Context::create();
        let mut codegen_ctx = CodegenContext::new(&context, "main");
        let mut codegen = Codegen::new(&mut codegen_ctx, &scope, &mut self.diagnostics);

        codegen.codegen_program(&items);

        if self.diagnostics.has_errors() {
            return Err(());
        }

        if self.output_ir {
            codegen_ctx.module.print_to_stderr();
        }

        Ok(())
    }
}

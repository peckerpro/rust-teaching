use std::{fs, path::Path};
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

    fn resolve_modules(items: &mut Vec<Item>, parent_path: &Path, diag: &mut DiagnosticBag, file: &Arc<SourceFile>) {
        for i in 0..items.len() {
            let has_body = match &items[i] {
                Item::Mod(m) => m.items.is_some(),
                _ => continue,
            };
            if has_body {
                continue;
            }

            let name = match &items[i] {
                Item::Mod(m) => m.name.clone(),
                _ => continue,
            };

            let mod_file = parent_path.join(format!("{}.rs", name));
            let mod_dir_file = parent_path.join(&name).join("mod.rs");

            let mod_path = if mod_file.exists() {
                mod_file
            } else if mod_dir_file.exists() {
                mod_dir_file
            } else {
                continue;
            };

            if let Ok(src) = fs::read_to_string(&mod_path) {
                let mod_file_name = mod_path.to_string_lossy().to_string();
                let mod_arc = Arc::new(SourceFile::new(mod_file_name.clone(), src.clone()));
                let lexer = Lexer::new(&src, mod_arc.clone());
                let mut parser = Parser::new(lexer, mod_arc);
                let mut mod_items = parser.parse_program();

                for d in parser.diagnostics.diagnostics() {
                    diag.error(d.message.clone(), d.span, file.clone());
                }
                if parser.diagnostics.has_errors() {
                    continue;
                }

                let mod_parent = mod_path.parent().unwrap_or(parent_path);
                Self::resolve_modules(&mut mod_items, mod_parent, diag, file);

                if let Item::Mod(ref mut m) = items[i] {
                    m.items = Some(mod_items);
                }
            }
        }
    }

    fn flatten_items(items: Vec<Item>) -> Vec<Item> {
        let mut result = Vec::new();
        for item in items {
            if let Item::Mod(m) = &item {
                if let Some(inner) = &m.items {
                    result.extend(Self::flatten_items(inner.clone()));
                }
            } else {
                result.push(item);
            }
        }
        result
    }

    pub fn compile(&mut self) -> Result<(), ()> {
        let src = self.file.src.clone();
        let file = self.file.clone();
        let parent_path = Path::new(&file.name).parent().unwrap_or(Path::new(".")).to_path_buf();

        let lexer = Lexer::new(&src, file.clone());
        let mut parser = Parser::new(lexer, file.clone());

        let mut items: Vec<Item> = parser.parse_program();

        if parser.diagnostics.has_errors() {
            for d in parser.diagnostics.diagnostics() {
                self.diagnostics.error(d.message.clone(), d.span, file.clone());
            }
            return Err(());
        }

        Self::resolve_modules(&mut items, &parent_path, &mut self.diagnostics, &file);

        let items = Self::flatten_items(items);

        if self.diagnostics.has_errors() {
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

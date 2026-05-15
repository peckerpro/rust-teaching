mod args;

use std::{fs, process};
use args::Args;
use clap::Parser as _;
use rt_common::diagnostic::{DiagnosticBag, Level};
use rt_common::span::SourceFile;
use rt_lexer::Lexer;
use rt_parser::Parser;
use rt_semantic::{resolve::NameResolver, typeck::TypeChecker};
use rt_codegen::{codegen::Codegen, context::CodegenContext};
use inkwell::context::Context;
use std::sync::Arc;
use ariadne::{Color, Label, Report, ReportKind, Source};

fn main() {
    let args = Args::parse();

    let source = match fs::read_to_string(&args.input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("rtc: error: cannot read '{}': {}", args.input, e);
            process::exit(1);
        }
    };

    let file = Arc::new(SourceFile::new(args.input.clone(), source.clone()));

    if args.lex_only {
        print_tokens(&source, &file);
        return;
    }

    if args.verbose { eprintln!("[stage 1/4] Lexing..."); }
    let lexer = Lexer::new(&source, file.clone());
    let mut par = Parser::new(lexer, file.clone());

    if args.verbose { eprintln!("[stage 2/4] Parsing..."); }
    let items = par.parse_program();
    if args.verbose { eprintln!("  => parsed {} items", items.len()); }

    if par.diagnostics.has_errors() {
        print_diagnostics_rich(&par.diagnostics, &file);
        process::exit(1);
    }

    if args.parse_only {
        if args.emit.as_deref() == Some("ast") {
            println!("{:#?}", items);
        } else {
            println!("parsed {} items successfully", items.len());
        }
        return;
    }

    let mut diag = par.diagnostics;
    if args.verbose { eprintln!("[stage 3/4] Semantic analysis..."); }
    let mut resolver = NameResolver::new(&mut diag, file.clone());
    let scope = resolver.resolve_program(&items);

    if diag.has_errors() {
        print_diagnostics_rich(&diag, &file);
        process::exit(1);
    }

    let mut typeck = TypeChecker::new(&scope, &mut diag, file.clone());
    typeck.check_program(&items);

    if diag.has_errors() {
        print_diagnostics_rich(&diag, &file);
        process::exit(1);
    }

    if args.verbose { eprintln!("  => type check passed"); }

    if args.check_only {
        eprintln!("type check passed");
        return;
    }

    if args.verbose { eprintln!("[stage 4/4] Code generation..."); }
    let context = Context::create();
    let mut codegen_ctx = CodegenContext::new(&context, &args.input);
    let mut codegen = Codegen::new(&mut codegen_ctx, &scope, &mut diag);
    codegen.codegen_program(&items);

    if diag.has_errors() {
        print_diagnostics_rich(&diag, &file);
        process::exit(1);
    }

    if args.verbose { eprintln!("  => LLVM IR generated"); }

    if args.emit_ir || args.emit.as_deref() == Some("ir") {
        let ir = codegen_ctx.module.print_to_string().to_string();
        if let Some(ref out) = args.output {
            fs::write(out, &ir).unwrap_or_else(|e| {
                eprintln!("rtc: error: cannot write '{}': {}", out, e);
                process::exit(1);
            });
        } else {
            print!("{}", ir);
        }
        return;
    }

    if args.compile_only || args.output.is_some() {
        eprintln!("rtc: note: object code generation not yet implemented; LLVM IR emitted instead.");
        let ir = codegen_ctx.module.print_to_string().to_string();
        let out_name = args.output.unwrap_or_else(|| {
            let mut s = args.input.clone();
            if s.ends_with(".rs") { s.truncate(s.len() - 3); }
            s + ".o"
        });
        fs::write(&out_name, ir).unwrap_or_else(|e| {
            eprintln!("rtc: error: cannot write '{}': {}", out_name, e);
            process::exit(1);
        });
    }

    eprintln!("compilation succeeded");
}

fn print_tokens(source: &str, file: &Arc<SourceFile>) {
    let lexer = Lexer::new(source, file.clone());
    for tok in lexer {
        let (line, col) = file.lookup_pos(tok.span.lo);
        println!("{}:{}: L:{:?} ({:?})", line, col, tok.kind, tok.span);
    }
}

fn print_diagnostics_rich(diag: &DiagnosticBag, file: &Arc<SourceFile>) {
    let mut builder = Report::build(ReportKind::Error, &file.name, 0);
    let mut has_any = false;

    for d in diag.diagnostics() {
        let (line, col) = file.lookup_pos(d.span.lo);
        let kind = match d.level {
            Level::Error => ReportKind::Error,
            Level::Warning => ReportKind::Warning,
            _ => ReportKind::Advice,
        };

        let label = Label::new((&file.name, d.span.lo.to_usize()..d.span.hi.to_usize()))
            .with_message(&d.message)
            .with_color(match d.level {
                Level::Error => Color::Red,
                Level::Warning => Color::Yellow,
                _ => Color::Blue,
            });

        builder = builder.with_label(label);
        if !has_any {
            builder = builder.with_message(format!("{}:{}: {}", line, col, d.message));
            has_any = true;
        }
    }

    builder
        .finish()
        .print((&file.name, Source::from(file.src.clone())))
        .unwrap();

    eprintln!("compilation failed with {} errors", diag.error_count());
}

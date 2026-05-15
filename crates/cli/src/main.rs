mod args;

use args::Args;
use clap::Parser;
use rt_driver::session::Session;
use std::fs;

fn main() {
    let args = Args::parse();

    let source = match fs::read_to_string(&args.input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error reading {}: {}", args.input, e);
            std::process::exit(1);
        }
    };

    let mut session = Session::new(source, args.input.clone(), args.emit_ir);

    match session.compile() {
        Ok(()) => {
            if session.diagnostics.diagnostics().is_empty() {
                eprintln!("compilation succeeded");
            }
        }
        Err(()) => {
            for d in session.diagnostics.diagnostics() {
                let (line, col) = session.file.lookup_pos(d.span.lo);
                eprintln!(
                    "{}:{}:{}: {}: {}",
                    session.file.name, line, col,
                    match d.level {
                        rt_common::diagnostic::Level::Error => "error",
                        rt_common::diagnostic::Level::Warning => "warning",
                        _ => "note",
                    },
                    d.message,
                );
            }
            eprintln!("compilation failed with {} errors", session.diagnostics.error_count());
            std::process::exit(1);
        }
    }
}

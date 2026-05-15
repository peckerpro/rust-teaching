use clap::Parser as ClapParser;

#[derive(ClapParser)]
#[command(
    name = "rtc",
    about = "Rust Teaching Compiler - a teaching-oriented Rust compiler",
    version = "0.1.0",
    after_help = "Examples:\n  \
                  rtc main.rs                  Compile to a.out\n  \
                  rtc main.rs -o myprog        Compile to myprog\n  \
                  rtc module.rs -c             Compile only, emit object file\n  \
                  rtc -S main.rs               Emit LLVM IR only\n  \
                  rtc --emit ast main.rs       Print AST\n  \
                  rtc --emit tokens main.rs    Print token stream\n  \
                  rtc --verbose main.rs        Show compilation stages"
)]
pub struct Args {
    #[arg(help = "Source file to compile")]
    pub input: String,

    #[arg(short = 'o', long, help = "Output file name (default: a.out)")]
    pub output: Option<String>,

    #[arg(short = 'c', long, help = "Compile only, do not link (emit .o)")]
    pub compile_only: bool,

    #[arg(short = 'S', long, help = "Emit LLVM IR (.ll), do not assemble")]
    pub emit_ir: bool,

    #[arg(short = 'O', long = "opt-level", help = "Optimization level (0-3)")]
    pub opt_level: Option<u8>,

    #[arg(short = 'v', long = "verbose", help = "Show compilation stages")]
    pub verbose: bool,

    #[arg(long = "emit", help = "Emit intermediate representation (tokens, ast, hir, ir)")]
    pub emit: Option<String>,

    #[arg(long = "check", short = 'C', help = "Type-check only, do not generate code")]
    pub check_only: bool,

    #[arg(long = "parse-only", help = "Parse only, do not check or generate")]
    pub parse_only: bool,

    #[arg(long = "lex-only", help = "Lex only, print token stream")]
    pub lex_only: bool,

    #[arg(long = "dump-scope", help = "Dump symbol table after name resolution")]
    pub dump_scope: bool,
}

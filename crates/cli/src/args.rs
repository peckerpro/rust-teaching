use clap::Parser as ClapParser;

#[derive(ClapParser)]
#[command(name = "rtc", about = "Rust Teaching Compiler")]
pub struct Args {
    #[arg(help = "Source file to compile")]
    pub input: String,

    #[arg(short, long, help = "Emit LLVM IR")]
    pub emit_ir: bool,

    #[arg(short, long, help = "Output file")]
    pub output: Option<String>,
}

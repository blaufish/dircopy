use std::fs;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: std::path::PathBuf,

    #[arg(short, long)]
    output: std::path::PathBuf,

    #[arg(short, long)]
    method: String,
}

fn copy_basic(input: std::path::PathBuf, output: std::path::PathBuf) {
    let fc = fs::copy(input, output);
    if let Err(e) = fc {
        eprintln!("Error: {}", e);
    }
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    match args.method.as_str() {
        "basic" => copy_basic(args.input, args.output),
        _ => eprintln!("Unimplemented: {}", args.method),
    };
    Ok(())
}

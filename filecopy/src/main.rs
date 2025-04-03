use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;

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

fn copy_own(input: std::path::PathBuf, output: std::path::PathBuf) {
    let fi_ = File::open(input);

    if let Err(e) = fi_ {
        eprintln!("Error: {}", e);
        return;
    }

    let fo_ = File::create(output);
    if let Err(e) = fo_ {
        eprintln!("Error: {}", e);
        return;
    }

    let mut fi = match fi_ {
        Ok( fi__ ) => fi__,
        Err( _ ) => panic!(),
    };
    let mut fo = match fo_ {
        Ok( fo__ ) => fo__,
        Err( _ ) => panic!(),
    };

    const BLOCK_SIZE : usize = 1024 * 1024;
    let mut buffer = [0u8; BLOCK_SIZE];
    loop {
        let fr_ = fi.read(&mut buffer[..]);
        if let Err(e) = fr_ {
            eprintln!("Error: {}", e);
            return;
        }
        let fr = match fr_ {
            Ok( fr__ ) => fr__,
            Err( _ ) => panic!(),
        };
        if fr == 0 {
            break;
        }
        let fw;
        if fr == BLOCK_SIZE {
            fw = fo.write(&buffer[..]);
        }
        else {
            fw = fo.write(&buffer[0..fr]);
        }
        if let Err(e) = fw {
            eprintln!("Error: {}", e);
            break;
        }
    }
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    match args.method.as_str() {
        "basic" => copy_basic(args.input, args.output),
        "own" => copy_own(args.input, args.output),
        _ => eprintln!("Unimplemented: {}", args.method),
    };
    Ok(())
}

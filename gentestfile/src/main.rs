use std::fs::File;
use std::io::Write;
use clap::Parser;
use sha2::{Sha256, Digest};

fn fill_array(seed: &[u8], length: usize) -> Box<[u8]> {
    let mut vec : Vec<u8> = Vec::with_capacity(length);
    let mut len = length;
    let mut first = true;
    let mut h1 = Sha256::new();
    h1.update(seed);
    let mut s = h1.finalize_reset();
    while len > 0 {
        if first {
            first = false;
        }
        else {
            h1.update(&s);
            s = h1.finalize_reset();
        }
        let slen = s.len();
        if len >= slen {
            vec.extend_from_slice(&s);
            len = len - slen;
        }
        else {
            let short = &s[0..len];
            vec.extend_from_slice(short);
        }
    }
    return vec.into_boxed_slice()
}

fn fill(mut file: File, seed: &[u8], length: usize) -> std::io::Result<()> {
    let array = fill_array(seed, length);
    return file.write_all(&array);
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    output: std::path::PathBuf,

    #[arg(short, long)]
    seed: String,

    #[arg(short, long)]
    length: usize,
}

fn main() {
    let args = Args::parse();
    let seed = args.seed.as_bytes();
    let path = args.output;
    let length = args.length;
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    let _ = match fill(file, seed, length) {
        Err(why) => panic!("Error writing {}: {}", display, why),
        Ok(_) => ()
    };
}

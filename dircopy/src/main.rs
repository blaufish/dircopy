use std::fs;
//use std::fs::File;
//use std::io::Read;
//use std::io::Write;
//use std::sync::mpsc::sync_channel;
//use std::thread;
use std::io;

use clap::Parser;
//use sha2::{Sha256, Digest};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: std::path::PathBuf,

    #[arg(short, long)]
    output: std::path::PathBuf,

    #[arg(short, long, default_value = "1M")]
    bs: String,

    #[arg(short, long, default_value_t = 10)]
    qs: usize,

}
/*
fn copy_sha256threaded(input: std::path::PathBuf, output: std::path::PathBuf) {
    const QUEUE_SIZE : usize = 10;
    const BLOCK_SIZE : usize = 1024 * 1024;
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

    let (read_tx, sha_rx) = sync_channel(QUEUE_SIZE);
    let (sha_tx, file_write_rx) = sync_channel(QUEUE_SIZE);

    let read_thread = thread::spawn(move || {
        loop {
            let mut buffer = [0u8; BLOCK_SIZE];
            match fi.read(&mut buffer) {
                Ok(0) => {
                    if let Err(e) = read_tx.send(Message::Done) {
                        eprintln!("Error: {}", e);
                    }
                    break;
                }
                Ok(n) => {
                    let block = buffer[0..n].to_vec();
                    if let Err(e) = read_tx.send(Message::Block(block)) {
                        eprintln!("Error: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
            }
        }
    });

    let sha_thread = thread::spawn( move || {
        let mut h1 = Sha256::new();
        loop {
            match sha_rx.recv() {
                Ok(Message::Block(block)) => {
                    h1.update(&block);
                    if let Err(e) = sha_tx.send(Message::Block(block)) {
                        eprintln!("Error: {}", e);
                    }
                }
                Ok(Message::Done) => {
                    if let Err(e) = sha_tx.send(Message::Done) {
                        eprintln!("Error: {}", e);
                    }
                    let digest = h1.clone().finalize();
                    print!("SHA: {}", format!("{:X}", digest));
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
            }
        }
    });

    let file_write_thread = thread::spawn( move || {
        loop {
            match file_write_rx.recv() {
                Ok(Message::Block(block)) => {
                    if let Err(e) = fo.write_all(&block) {
                        eprintln!("Error: {}", e);
                        break;
                    }
                }
                Ok(Message::Done) => {
                    break;
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
            }
        }
    });


    if let Err(_) = read_thread.join() {
        panic!("Failure to join read thread");
    }
    if let Err(_) = sha_thread.join() {
        panic!("Failure to join sha thread");
    }
    if let Err(_) = file_write_thread.join() {
        panic!("Failure to join file write thread");
    }
}
*/


fn copy_dir(input: std::path::PathBuf, _output: std::path::PathBuf) -> io::Result<()> {
    for entry in fs::read_dir(input)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            println!("path: {}", path.display());
            if let Err(e) = copy_dir(path, _output.clone()) {
              return Err(e);
            }
        }
    }
    Ok(())
}

fn s2i(string : String) -> usize {
    let mut prefix : usize = 0;
    let mut exponent : usize = 1;
    for c in string.chars() {
        match c {
            'K' => exponent = 1024,
            'M' => exponent = 1024 * 1024,
            'G' => exponent = 1024 * 1024 * 1024,
            '0' => prefix = prefix * 10,
            '1' => prefix = prefix * 10 + 1,
            '2' => prefix = prefix * 10 + 2,
            '3' => prefix = prefix * 10 + 3,
            '4' => prefix = prefix * 10 + 4,
            '5' => prefix = prefix * 10 + 5,
            '6' => prefix = prefix * 10 + 6,
            '7' => prefix = prefix * 10 + 7,
            '8' => prefix = prefix * 10 + 8,
            '9' => prefix = prefix * 10 + 9,
            _ => eprintln!("Unable to parse: {}", string)
        }
    }
    let result = prefix * exponent;
    if result < 1 {
       eprintln!("Unable to parse: {}", string)
    }
    return result;
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let blocksize : usize = s2i(args.bs);

    if blocksize < 1 {
        return Ok(())
    }
    eprintln!("blocksize: {}", blocksize);

    if ! args.input.is_dir() {
        eprintln!("Directory {} is not a directory", args.input.display());
        return Ok(())
    }

    //if ! args.output.is_dir() {
    //    eprintln!("Directory {} is not a directory", args.output.display());
    //    return Ok(())
    //}

    if let Err(e) = copy_dir(args.input, args.output) {
        eprintln!("copy_dir failed: {}", e);
        return Err(e);
    }

    Ok(())
}

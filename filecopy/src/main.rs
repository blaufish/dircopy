use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::sync::mpsc::sync_channel;
use std::thread;

//use std::sync::Arc;
//use std::sync::Mutex;
//use std::time;
//use std::collections::VecDeque;

use clap::Parser;
use sha2::{Sha256, Digest};

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

fn copy_sha256(input: std::path::PathBuf, output: std::path::PathBuf) {
    let mut h1 = Sha256::new();

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
            h1.update(&buffer[..]);
            fw = fo.write(&buffer[..]);
        }
        else {
            h1.update(&buffer[0..fr]);
            fw = fo.write(&buffer[0..fr]);
        }
        if let Err(e) = fw {
            eprintln!("Error: {}", e);
            break;
        }
    }
    print!("SHA: {}", format!("{:X}", h1.finalize()))
}


enum Message {
    Block(Vec<u8>),
    Done,
}

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

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    match args.method.as_str() {
        "basic" => copy_basic(args.input, args.output),
        "own" => copy_own(args.input, args.output),
        "sha256" => copy_sha256(args.input, args.output),
        "sha256mt" => copy_sha256threaded(args.input, args.output),
        _ => eprintln!("Unimplemented: {}", args.method),
    };
    Ok(())
}

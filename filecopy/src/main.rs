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

#[derive(PartialEq)]
enum Op {
    OpInit,
    OpBlock,
    OpFinalize,
    OpExit,
}
/*
struct Msg {
    op: Op,
}
impl Msg {
    fn new( op: Op ) -> Msg {
        Msg { op: op }
    }
}
*/

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

    let (sha_send, sha_rec) = sync_channel(QUEUE_SIZE);
    let (sha_send_data, sha_rec_data) = sync_channel(QUEUE_SIZE);
    let (write_send, write_rec) = sync_channel(QUEUE_SIZE);
    let (write_send_data, write_rec_data) = sync_channel(QUEUE_SIZE);

    let mut buffer = [0u8; BLOCK_SIZE];
    let sha_thread = thread::spawn( move || {
        let mut h1 = Sha256::new();
        loop {
            let op: Op = sha_rec.recv().unwrap();
            if op == Op::OpInit {
                //print!("init");
                h1.reset();
                continue;
            }
            if op == Op::OpBlock {
                //print!("block");
                let data : Vec<u8> = sha_rec_data.recv().unwrap();
                h1.update(data.as_slice());
                continue;
            }
            if op == Op::OpFinalize {
                //print!("fin");
                let digest = h1.clone().finalize();
                print!("SHA: {}", format!("{:X}", digest));
                continue;
            }
            if op == Op::OpExit {
                //print!("exit");
                break;
            }
        }
    });

    let file_write_thread = thread::spawn( move || {
        loop {
            let op: Op = write_rec.recv().unwrap();
            if op == Op::OpBlock {
                //print!("block");
                let data : Vec<u8> = write_rec_data.recv().unwrap();
                let _ = fo.write(data.as_slice());
                continue;
            }
            if op == Op::OpExit {
                //print!("exit");
                break;
            }
        }
    });

    if let Err(e) = sha_send.send(Op::OpInit) {
        panic!("{}", e);
    }
    if let Err(e) = write_send.send(Op::OpInit) {
        panic!("{}", e);
    }

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
            let _ = sha_send.send(Op::OpFinalize);
            let _ = write_send.send(Op::OpFinalize);
            break;
        }
        let _ = sha_send.send(Op::OpBlock);
        let _ = write_send.send(Op::OpBlock);
        let mut vector : Vec<u8> = Vec::with_capacity(fr);
        vector.extend_from_slice(&buffer[0..fr]);
        let _ = sha_send_data.send(vector.clone());
        let _ = write_send_data.send(vector);
    }
    if let Err(e) = sha_send.send(Op::OpExit) {
        panic!("{}", e);
    }
    if let Err(e) = write_send.send(Op::OpExit) {
        panic!("{}", e);
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

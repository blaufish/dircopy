use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::sync::mpsc::sync_channel;
use std::thread;
use std::io;

use clap::Parser;
use sha2::{Sha256, Digest};

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

#[derive(Clone,Copy)]
struct Configuration {
    queue_size : usize,
    block_size : usize,
}

enum Message {
    Block(Vec<u8>),
    Done,
}

fn copy(cfg: Configuration, input: std::path::PathBuf, output: std::path::PathBuf) -> Result<String, String> {
    const BLOCK_SIZE : usize = 1024 * 1024;
    const QUEUE_SIZE : usize = 10;
    //cfg.queue_size;

    let mut result : String = "".to_string();

    let fi_ = File::open(input);

    if let Err(e) = fi_ {
        eprintln!("Error: {}", e);
        return Err(result); // TODO return a proper cause
    }

    let fo_ = File::create(output);
    if let Err(e) = fo_ {
        eprintln!("Error: {}", e);
        return Err(result); //TODO return a proper cause
    }

    let mut fi = match fi_ {
        Ok( fi__ ) => fi__,
        Err( _ ) => panic!(),
    };
    let mut fo = match fo_ {
        Ok( fo__ ) => fo__,
        Err( _ ) => panic!(),
    };

    let (sha_tx, sha_rx) = sync_channel::<Message>(QUEUE_SIZE);
    let (file_write_tx, file_write_rx) = sync_channel::<Message>(QUEUE_SIZE);
    let (hash_tx, hash_rx) = sync_channel::<String>(1);

    let read_thread = thread::spawn(move || {
        loop {
            let mut buffer = [0u8; BLOCK_SIZE];
            match fi.read(&mut buffer) {
                Ok(0) => {
                    break;
                }
                Ok(n) => {
                    let mut err = false;
                    if let Err(e) = sha_tx.send(Message::Block(buffer[0..n].to_vec())) {
                        eprintln!("Error: {}", e);
                        err = true;
                    }
                    if let Err(e) = file_write_tx.send(Message::Block(buffer[0..n].to_vec())) {
                        eprintln!("Error: {}", e);
                        err = true;
                    }
                    if err {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    break;
                }
            }
        }
        if let Err(e) = sha_tx.send(Message::Done) {
            eprintln!("Error: {}", e);
        }
        if let Err(e) = file_write_tx.send(Message::Done) {
            eprintln!("Error: {}", e);
        }
    });

    let sha_thread = thread::spawn( move || {
        let mut h1 = Sha256::new();
        let mut incomplete = true;
        loop {
            match sha_rx.recv() {
                Ok(Message::Block(block)) => {
                    h1.update(&block);
                }
                Ok(Message::Done) => {
                    let digest = h1.clone().finalize();
                    let strdigest = format!("{:X}", digest);
                    if let Err(e) = hash_tx.send(strdigest) {
                        eprintln!("Error T-SHA: {}", e);
                    }
                    incomplete = false;
                    break;
                }
                Err(e) => {
                    eprintln!("Error T-SHA: {}", e);
                    break;
                }
            }
        }
        if incomplete {
            if let Err(e) = hash_tx.send("".to_string()) {
                eprintln!("Error T-SHA: {}", e);
            }
        }
    });

    let file_write_thread = thread::spawn( move || {
        loop {
            match file_write_rx.recv() {
                Ok(Message::Block(block)) => {
                    if let Err(e) = fo.write_all(&block) {
                        eprintln!("Error T-FW: {}", e);
                        break;
                    }
                }
                Ok(Message::Done) => {
                    break;
                }
                Err(e) => {
                    eprintln!("Error T-FW: {}", e);
                    break;
                }
            }
        }
    });


    let mut failed = true;

    match hash_rx.recv() {
        Ok(s) => {
            if s.len() == 64 {
                result = s;
                failed = false;
            }
            else {
                eprintln!("Bad SHA-256 received: '{}'", s);
            }
        },
        Err(e) => {
            eprintln!("Error receiving sha256: {}", e);
        }
    }

    if let Err(_) = read_thread.join() {
        panic!("Failure to join read thread");
    }
    if let Err(_) = sha_thread.join() {
        panic!("Failure to join sha thread");
    }
    if let Err(_) = file_write_thread.join() {
        panic!("Failure to join file write thread");
    }

    if failed {
        return Err("failed".to_string());
    }
    Ok(result)
}


fn copy_directory(
    cfg: Configuration,
    input: std::path::PathBuf,
    output: std::path::PathBuf) -> io::Result<()> {
    let rel = std::path::PathBuf::new();
    return copy_dir(cfg, input, rel, output);
}

fn copy_dir(
    cfg: Configuration,
    input: std::path::PathBuf,
    rel:  std::path::PathBuf,
    output: std::path::PathBuf) -> io::Result<()> {
    for entry in fs::read_dir(input)? {
        let entry = entry?;
        let path = entry.path();
        let mut output_path = output.clone();
        let mut rel2 = rel.clone();
        match path.file_name() {
            Some(s) => {
                output_path.push(s);
                rel2.push(s);
            },
            None => continue, //TODO error handling
        }
        if path.is_dir() {
            println!("dir: {} --> {}", path.display(), output_path.display());
            if !output_path.exists() {
                fs::create_dir(output_path.clone())?;
            }
            copy_dir(cfg, path, rel2, output_path)?;
        }
        else if path.is_file() {
            //println!("file: {}", path.display());
            //println!("file out: {}", output_path.clone().display());
            match copy(cfg, path, output_path) {
                Ok(s) => {
                    println!("{}  {}", s.to_lowercase(), rel2.display());
                },
                Err(_s) => {
                    return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof));
                }
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

    let cfg = Configuration {
        queue_size: args.qs,
        block_size: blocksize,
    };

    if ! args.input.is_dir() {
        eprintln!("Directory {} is not a directory", args.input.display());
        return Ok(())
    }

    if ! args.output.is_dir() {
        eprintln!("Directory {} is not a directory", args.output.display());
        return Ok(())
    }

    if let Err(e) = copy_directory(cfg, args.input, args.output) {
        eprintln!("copy_dir failed: {}", e);
        return Err(e);
    }

    Ok(())
}

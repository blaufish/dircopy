use std::fs;
//use std::fs::File;
//use std::fs::OpenOptions;
//use std::io;
//use std::io::IsTerminal;
//use std::io::Read;
//use std::io::Write;
//use std::sync::mpsc::sync_channel;
//use std::thread;
//use std::time::Instant;

use std::process::ExitCode;

//use chrono::prelude::*;
use clap::Parser;
//use sha2::{Digest, Sha256};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    dir: Vec<std::path::PathBuf>,

}

fn main() -> ExitCode {
    let args = Args::parse();

    if args.dir.len() == 0 {
        eprintln!("Error: No directory specified");
    }

    let mut sha_files : Vec<(std::path::PathBuf, Vec<String>)> = Vec::new();
    for dir in args.dir {
        if !dir.is_dir() {
            eprintln!("Error: Not a directory {}", dir.display());
            return ExitCode::from(1);
        }

        let read_dir_maybe = fs::read_dir(&dir);
        let read_dir;
        match read_dir_maybe {
            Ok(rd) => read_dir = rd,
            Err(e) => {
                eprintln!("Error indexing {}: {}", dir.display(), e);
                return ExitCode::from(1);
            }
        }
        let mut names : Vec<String> = Vec::new();
        for entry in read_dir {
            let name;
            match entry {
                Ok(file_entry) => {
                    match file_entry.file_name().into_string() {
                        Ok(n) => name = n,
                        Err(e) => {
                            eprintln!("Error reading file name: {}", e.display());
                            return ExitCode::from(1);
                        }
                    }
                    match file_entry.file_type() {
                        Ok(file_type) => {
                            if !file_type.is_file() {
                                continue;
                            }
                        }
                        Err(e) => {
                            eprintln!("Error determining file type; {} {}", name, e);
                            return ExitCode::from(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error, unexpected: {}", e);
                    return ExitCode::from(1);
                }
            }
            if !name.starts_with("shasum.") {
                continue;
            }
            if !name.ends_with(".txt") {
                continue;
            }
            //println!("... {}", name);
            names.push(name);
        }
        sha_files.push((dir, names));
    }

    for (dir, names) in sha_files {
        for name in names {
            println!("Found files: {} - {}", dir.display(), name);
        }
    }

    ExitCode::SUCCESS
}


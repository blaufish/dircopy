use std::fs;
use std::fs::File;
//use std::fs::OpenOptions;
//use std::io;
use std::io::BufRead;
use std::io::BufReader;
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

fn parse_line(line: String) -> Result<(String, String), String> {
    if line.len() < 67 {
        return Err(String::from("Too short"));
    }
    match &line[64..66] {
        "  " => (),
        _ => return Err(String::from("Expected 2 spaces")),
    }
    let hash = &line[..64];
    let filename = &line[66..];

    Ok((hash.to_string(), filename.to_string()))
}

fn verify_list(dir: &std::path::PathBuf, list: std::path::PathBuf) -> ExitCode {
    let file;
    match File::open(&list) {
        Ok(f) => file = f,
        Err(e) => {
            eprintln!("Error opening {}: {}", list.display(), e);
            return ExitCode::from(2);
        }
    }
    let reader = BufReader::new(file);
    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                println!("debug...{} {}", list.display(), line);
                match parse_line(line) {
                    Ok((hash, filename)) => {
                        println!("debug... hash: {}", hash);
                        println!("debug... filename: {}", filename);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return ExitCode::from(2);
                    }
                }
            }
            Err(e) => {
                eprintln!("Unexpected error processing {}: {}", list.display(), e);
                return ExitCode::from(2);
            }
        }
    }
    ExitCode::SUCCESS
}

fn verify_all_lists(dir: &std::path::PathBuf, sha_files: Vec<String>) -> ExitCode {
    for sha_file in sha_files {
        let mut sha_file_pb = dir.clone();
        sha_file_pb.push(sha_file);
        let e = verify_list(dir, sha_file_pb);
        if e != ExitCode::SUCCESS {
            return e;
        }
    }
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let args = Args::parse();

    if args.dir.len() == 0 {
        eprintln!("Error: No directory specified");
    }

    let mut sha_files: Vec<(std::path::PathBuf, Vec<String>)> = Vec::new();
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
        let mut names: Vec<String> = Vec::new();
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
            names.push(name);
        }
        if names.len() == 0 {
            eprintln!("Error: no shasum.*.txt files in {}", dir.display());
            return ExitCode::from(1);
        }
        sha_files.push((dir, names));
    }

    for (dir, names) in sha_files {
        for name in names.clone() {
            println!("Found files: {} - {}", dir.display(), name);
        }
        let exit_code = verify_all_lists(&dir, names);
        if exit_code != ExitCode::SUCCESS {
            return exit_code;
        }
    }

    ExitCode::SUCCESS
}

use std::fs;
use std::fs::File;
//use std::fs::OpenOptions;
//use std::io;
use std::io::BufRead;
use std::io::BufReader;
//use std::io::IsTerminal;
use std::io::Read;
//use std::io::Write;
use std::path::MAIN_SEPARATOR_STR;
use std::process::ExitCode;
//use std::sync::mpsc::sync_channel;
//use std::thread;
use std::time::Instant;

//use chrono::prelude::*;
use clap::Parser;
use sha2::{Digest, Sha256};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    dir: Vec<std::path::PathBuf>,

    #[arg(long, conflicts_with = "convert_paths")]
    no_convert_paths: bool,

    #[arg(long)]
    convert_paths: bool,
}

fn parse_line(line: String, convert_paths: bool) -> Result<(String, String), String> {
    if line.len() < 67 {
        return Err(String::from("Too short"));
    }
    match &line[64..66] {
        "  " => (),
        _ => return Err(String::from("Expected 2 spaces")),
    }
    let hash = &line[..64];
    let filename = &line[66..];
    let filename_corrected;

    if convert_paths {
        if !filename.contains(MAIN_SEPARATOR_STR) {
            match MAIN_SEPARATOR_STR {
                "\\" => filename_corrected = filename.replace("/", "\\"),
                "/" => filename_corrected = filename.replace("\\", "/"),
                &_ => filename_corrected = filename.to_string(),
            }
        } else {
            filename_corrected = filename.to_string();
        }
    } else {
        filename_corrected = filename.to_string();
    }

    Ok((hash.to_string(), filename_corrected))
}

fn sha_file(file: &mut File) -> Result<String, String> {
    let block_size: usize = 128 * 1024;
    let mut h1 = Sha256::new();

    let mut heap_buf: Vec<u8> = Vec::with_capacity(block_size);
    heap_buf.resize(block_size, 0x00);

    loop {
        match file.read(&mut heap_buf[0..block_size]) {
            Ok(0) => break,
            Ok(n) => h1.update(&heap_buf[0..n]),
            Err(e) => {
                return Err(e.to_string());
            }
        }
    }
    let digest = h1.finalize();
    let strdigest = format!("{:x}", digest);
    Ok(strdigest)
}

fn verify_file(file_path: std::path::PathBuf, hash: String) -> ExitCode {
    let mut file: File;
    match File::open(&file_path) {
        Ok(file_) => file = file_,
        Err(e) => {
            eprintln!(
                "Unexpected error opening file {}: {}",
                file_path.display(),
                e
            );
            return ExitCode::from(3);
        }
    }
    match sha_file(&mut file) {
        Ok(strdigest) => {
            if hash == strdigest {
                println!("{}: OK", file_path.display());
            } else {
                println!("{}: FAILED (mismatch)", file_path.display());
            }
        }
        Err(err) => {
            println!("{}: FAILED (error: {})", file_path.display(), err);
        }
    }

    ExitCode::SUCCESS
}

fn verify_list(
    dir: &std::path::PathBuf,
    list: std::path::PathBuf,
    convert_paths: bool,
) -> ExitCode {
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
                //println!("debug...{} {}", list.display(), line);
                match parse_line(line, convert_paths) {
                    Ok((hash, filename)) => {
                        //println!("debug... hash: {}", hash);
                        //println!("debug... filename: {}", filename);
                        let mut file_path = dir.clone();
                        file_path.push(filename);
                        let e = verify_file(file_path, hash);
                        if e != ExitCode::SUCCESS {
                            return e;
                        }
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

fn verify_all_lists(
    dir: &std::path::PathBuf,
    sha_files: Vec<String>,
    convert_paths: bool,
) -> ExitCode {
    for sha_file in sha_files {
        let mut sha_file_pb = dir.clone();
        sha_file_pb.push(sha_file);
        let e = verify_list(dir, sha_file_pb, convert_paths);
        if e != ExitCode::SUCCESS {
            return e;
        }
    }
    ExitCode::SUCCESS
}

fn op(yes_flag: bool, no_flag: bool, default_if_not_specifed: bool) -> bool {
    if yes_flag {
        true
    } else if no_flag {
        false
    } else {
        default_if_not_specifed
    }
}

fn inspect_dir(dir: &std::path::PathBuf) -> Result<Vec<String>, String> {
    if !dir.is_dir() {
        return Err(format!("Not a directory {}", dir.display()));
    }
    let read_dir_maybe = fs::read_dir(&dir);
    let read_dir;
    match read_dir_maybe {
        Ok(rd) => read_dir = rd,
        Err(e) => {
            return Err(format!("{}: {}", dir.display(), e));
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
                        return Err(format!("Error reading file name: {}", e.display()));
                    }
                }
                match file_entry.file_type() {
                    Ok(file_type) => {
                        if !file_type.is_file() {
                            continue;
                        }
                    }
                    Err(e) => {
                        return Err(format!("Error determining file type; {} {}", name, e));
                    }
                }
            }
            Err(e) => {
                return Err(format!("Unexpected: {}", e));
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
    Ok(names)
}

fn main() -> ExitCode {
    let args = Args::parse();

    if args.dir.len() == 0 {
        eprintln!("Error: No directory specified");
    }

    let mut sha_files: Vec<(std::path::PathBuf, Vec<String>)> = Vec::new();
    for dir in args.dir {
        match inspect_dir(&dir) {
            Ok(names) => {
                if names.len() == 0 {
                    eprintln!("Error: no shasum.*.txt files in {}", dir.display());
                    return ExitCode::from(1);
                }
                sha_files.push((dir, names));
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                return ExitCode::from(1);
            }
        }
    }

    let convert_paths = op(args.convert_paths, args.no_convert_paths, true);

    let start = Instant::now();

    for (dir, names) in sha_files {
        for name in names.clone() {
            println!("Found files: {} - {}", dir.display(), name);
        }
        let exit_code = verify_all_lists(&dir, names, convert_paths);
        if exit_code != ExitCode::SUCCESS {
            return exit_code;
        }
    }
    let seconds = start.elapsed().as_secs();
    println!("Execution time: {}s", seconds);

    ExitCode::SUCCESS
}

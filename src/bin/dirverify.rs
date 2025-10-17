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
use std::sync::mpsc::sync_channel;
use std::thread;
use std::time::Instant;

//use chrono::prelude::*;
use clap::Parser;
use sha2::{Digest, Sha256};

/// A directory verifier. Searches for shasum*.txt files in directories.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Directories with files to be verified
    dir: Vec<std::path::PathBuf>,

    /// Specify sha256-file and disable shasum*.txt
    #[arg(long)]
    hash_file: Option<std::path::PathBuf>,

    /// Keep paths exactly as is. Do not try to workaround unix, dos mismatches.
    #[arg(long)]
    no_convert_paths: bool,

    /// Do not print a summary
    #[arg(long)]
    no_summary: bool,

    /// Disable threaded sha read/hash behavior
    #[arg(long)]
    no_threaded_sha: bool,

    /// Print informative messages helpful for understanding processing
    #[arg(long)]
    verbose: bool,
}

struct Statistics {
    read_bytes: usize,
    read_files: usize,
    matches: usize,
    mismatches: usize,
    errors: usize,
}

impl Statistics {
    fn new() -> Statistics {
        Statistics {
            read_bytes: 0,
            read_files: 0,
            matches: 0,
            mismatches: 0,
            errors: 0,
        }
    }
}

struct DirVerify {
    convert_paths: bool,
    threaded_sha_reader: bool,
}

impl DirVerify {
    fn parse_line(&self, line: String) -> Result<(String, String), String> {
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

        if self.convert_paths {
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

    fn verify_list(
        &self,
        stats: &mut Statistics,
        dir: &std::path::PathBuf,
        list: &std::path::PathBuf,
    ) {
        let file;
        match File::open(&list) {
            Ok(f) => file = f,
            Err(e) => {
                eprintln!("Error opening {}: {}", list.display(), e);
                stats.errors += 1;
                return;
            }
        }
        let reader = BufReader::new(file);
        for line_result in reader.lines() {
            match line_result {
                Ok(line) => match self.parse_line(line) {
                    Ok((hash, filename)) => {
                        let mut file_path = dir.clone();
                        file_path.push(filename);
                        self.verify_file(stats, file_path, hash);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        stats.errors += 1;
                        return;
                    }
                },
                Err(e) => {
                    eprintln!("Unexpected error processing {}: {}", list.display(), e);
                    stats.errors += 1;
                }
            }
        }
    }

    fn verify_all_lists(
        &self,
        stats: &mut Statistics,
        dir: &std::path::PathBuf,
        sha_files: &Option<Vec<String>>,
        sha_file: &Option<std::path::PathBuf>,
    ) {
        if let Some(files) = sha_files {
            for file in files {
                let mut sha_file_pb = dir.clone();
                sha_file_pb.push(file);
                self.verify_list(stats, dir, &sha_file_pb);
            }
        }
        if let Some(file) = sha_file {
            self.verify_list(stats, dir, &file);
        }
    }

    fn verify_file(&self, stats: &mut Statistics, file_path: std::path::PathBuf, hash: String) {
        let mut file: File;
        match File::open(&file_path) {
            Ok(file_) => file = file_,
            Err(e) => {
                eprintln!(
                    "Unexpected error opening file {}: {}",
                    file_path.display(),
                    e
                );
                stats.errors += 1;
                return;
            }
        }
        stats.read_files += 1;
        match self.sha_file(stats, &mut file) {
            Ok(strdigest) => {
                if hash == strdigest {
                    println!("{}: OK", file_path.display());
                    stats.matches += 1;
                } else {
                    println!("{}: FAILED (mismatch)", file_path.display());
                    stats.mismatches += 1;
                }
            }
            Err(err) => {
                println!("{}: FAILED (error: {})", file_path.display(), err);
                stats.errors += 1;
            }
        }
    }

    fn sha_file(&self, stats: &mut Statistics, file: &mut File) -> Result<String, String> {
        if self.threaded_sha_reader {
            sha_file_multithread(stats, file)
        } else {
            sha_file_single_thread(stats, file)
        }
    }
}

fn sha_file_single_thread(stats: &mut Statistics, file: &mut File) -> Result<String, String> {
    let block_size: usize = 128 * 1024;
    let mut h1 = Sha256::new();

    let mut heap_buf: Vec<u8> = Vec::with_capacity(block_size);
    heap_buf.resize(block_size, 0x00);

    loop {
        match file.read(&mut heap_buf[0..block_size]) {
            Ok(0) => break,
            Ok(n) => {
                h1.update(&heap_buf[0..n]);
                stats.read_bytes += n;
            }
            Err(e) => {
                return Err(e.to_string());
            }
        }
    }
    let digest = h1.finalize();
    let strdigest = format!("{:x}", digest);
    Ok(strdigest)
}

enum Message {
    Block(Vec<u8>),
    Done,
    Error,
}

fn sha_file_multithread(stats: &mut Statistics, file: &mut File) -> Result<String, String> {
    let block_size: usize = 128 * 1024;
    let queue_size: usize = 2;

    let (read_tx, sha_rx) = sync_channel::<Message>(queue_size);

    let sha_thread = thread::spawn(move || -> Result<String, String> {
        let mut h1 = Sha256::new();
        loop {
            match sha_rx.recv() {
                Ok(Message::Block(block)) => {
                    h1.update(&block);
                }
                Ok(Message::Error) => {
                    return Err(String::from("T-Read: sent error"));
                }
                Ok(Message::Done) => {
                    break;
                }
                Err(e) => {
                    return Err(format!("T-SHA: {}", e));
                }
            }
        }
        let digest = h1.finalize();
        let strdigest = format!("{:x}", digest);
        return Ok(strdigest);
    });

    let mut heap_buf: Vec<u8> = Vec::with_capacity(block_size);
    heap_buf.resize(block_size, 0x00);

    loop {
        match file.read(&mut heap_buf[0..block_size]) {
            Ok(0) => {
                if let Err(e) = read_tx.send(Message::Done) {
                    return Err(format!("Error: {}", e));
                }
                break;
            }
            Ok(n) => {
                stats.read_bytes += n;
                if let Err(e) = read_tx.send(Message::Block(heap_buf[0..n].to_vec())) {
                    return Err(format!("Error: {}", e));
                }
            }
            Err(e) => {
                _ = read_tx.send(Message::Error);
                return Err(e.to_string());
            }
        }
    }
    match sha_thread.join() {
        Ok(x) => x,
        Err(err) => Err(format!("Join error: {:?}", err)),
    }
}

fn inspect_dir(dir: &std::path::PathBuf, detect_sha_files: bool) -> Result<Vec<String>, String> {
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
    if !detect_sha_files {
        return Ok(Vec::new());
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
        return ExitCode::from(1);
    }

    let mut sha_files: Vec<(std::path::PathBuf, Vec<String>)> = Vec::new();
    for dir in args.dir {
        match inspect_dir(&dir, args.hash_file.is_none()) {
            Ok(names) => {
                if args.hash_file.is_none() {
                    if names.len() == 0 {
                        eprintln!("Error: no shasum.*.txt files in {}", dir.display());
                        return ExitCode::from(1);
                    }
                    sha_files.push((dir, names));
                } else {
                    //Names doesn't matter at all in this codepath...
                    //TODO: Make Option, return None, for clarity
                    sha_files.push((dir, names));
                }
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                return ExitCode::from(1);
            }
        }
    }

    let dirverify = DirVerify {
        convert_paths: !args.no_convert_paths,
        threaded_sha_reader: !args.no_threaded_sha,
    };

    if args.verbose {
        for (dir, names) in sha_files.clone() {
            for name in names.clone() {
                println!("Found files: {} - {}", dir.display(), name);
            }
        }
        if let Some(ref hash_file) = args.hash_file {
            println!("Utilizing specified shasum file: {}", hash_file.display());
        }
    }

    let mut stats = Statistics::new();
    let start = Instant::now();

    // ------ run the verifier ------
    let hash_file = args.hash_file;
    for (dir, names) in &sha_files {
        let hash_names = match hash_file {
            Some(_) => None,
            None => Some(names.clone()),
        };
        dirverify.verify_all_lists(&mut stats, &dir, &hash_names, &hash_file);
    }
    // ------

    if !args.no_summary {
        let seconds = start.elapsed().as_secs();
        println!("Summary:");
        println!("* Execution time: {}s", seconds);
        println!("* Read (files): {}", stats.read_files);
        println!("* Read (bytes): {}", stats.read_bytes);
        println!("* Files matching: {}", stats.matches);
        println!("* Files mismatching: {}", stats.mismatches);
        println!("* Errors: {}", stats.errors);
    }
    if stats.errors != 0 {
        return ExitCode::from(1);
    }
    if stats.mismatches != 0 {
        return ExitCode::from(1);
    }

    ExitCode::SUCCESS
}

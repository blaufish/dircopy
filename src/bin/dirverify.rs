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

    /// Do not check multiple directories at the same time
    #[arg(long)]
    no_parallell: bool,

    /// Size of queue between reader and hasher thread. Tuning parameter.
    #[arg(long, default_value_t = 2)]
    queue_size: usize,

    /// Size of blocks between reader and hasher thread. Tuning parameter.
    #[arg(long, default_value = "128K")]
    block_size: String,

    /// Print informative messages helpful for understanding processing
    #[arg(long)]
    verbose: bool,
}

enum Message {
    Block(Vec<u8>),
    Done,
    Error,
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
    fn add(&mut self, other: &Statistics) {
        self.read_bytes += other.read_bytes;
        self.read_files += other.read_files;
        self.matches += other.matches;
        self.mismatches += other.mismatches;
        self.errors += other.errors;
    }
}

#[derive(Clone)]
struct DirVerify {
    convert_paths: bool,
    threaded_sha_reader: bool,
    block_size: usize,
    queue_size: usize,
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
            self.sha_file_multithread(stats, file)
        } else {
            self.sha_file_single_thread(stats, file)
        }
    }

    fn sha_file_single_thread(
        &self,
        stats: &mut Statistics,
        file: &mut File,
    ) -> Result<String, String> {
        let block_size = self.block_size;
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

    fn sha_file_multithread(
        &self,
        stats: &mut Statistics,
        file: &mut File,
    ) -> Result<String, String> {
        let block_size = self.block_size;
        let queue_size = self.queue_size;

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

// Convert "128K" into 128*1024, and such
fn s2i(string: String) -> usize {
    let mut prefix: usize = 0;
    let mut exponent: usize = 1;
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
            _ => eprintln!("Unable to parse: {}", string),
        }
    }
    let result = prefix * exponent;
    if result < 1 {
        eprintln!("Unable to parse: {}", string)
    }
    return result;
}

fn bandwidth(read_bytes: usize, seconds: u64) -> String {
    if seconds == 0 {
        return String::from("NaN");
    }
    let mut rb = (read_bytes as f64) / (seconds as f64);
    let sufixes: Vec<&str> = vec!["B", "KB", "MB", "GB", "TB", "PB"];
    let mut suff = "";
    for s in sufixes {
        suff = s;
        if rb < 1000.0 {
            break;
        }
        rb = rb / 1000.0;
    }
    return format!("{:.3} {}/s", rb, suff);
}

fn run_parallell(
    dirverify: DirVerify,
    hash_file: Option<std::path::PathBuf>,
    sha_files: Vec<(std::path::PathBuf, Vec<String>)>,
) -> Statistics {
    let mut stats = Statistics::new();
    let mut threads = Vec::new();
    for (dir, names) in &sha_files {
        let hash_file = hash_file.clone();
        let dir_thread = dir.clone();
        let names_thread = names.clone();
        let dirverify_thread = dirverify.clone();
        let thread = thread::spawn(move || -> Statistics {
            let mut thread_stats = Statistics::new();
            let hash_names = match hash_file {
                Some(_) => None,
                None => Some(names_thread.clone()),
            };
            dirverify_thread.verify_all_lists(
                &mut thread_stats,
                &dir_thread,
                &hash_names,
                &hash_file,
            );
            thread_stats
        });
        threads.push(thread);
    }
    for thread in threads {
        match thread.join() {
            Ok(x) => stats.add(&x),
            Err(err) => {
                stats.errors += 1;
                eprintln!("{}", format!("Join error: {:?}", err));
            }
        }
    }
    stats
}

fn run_sequential(
    dirverify: DirVerify,
    hash_file: Option<std::path::PathBuf>,
    sha_files: Vec<(std::path::PathBuf, Vec<String>)>,
) -> Statistics {
    let mut stats = Statistics::new();
    for (dir, names) in &sha_files {
        let hash_names = match hash_file {
            Some(_) => None,
            None => Some(names.clone()),
        };
        dirverify.verify_all_lists(&mut stats, &dir, &hash_names, &hash_file);
    }
    stats
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
        block_size: s2i(args.block_size),
        queue_size: args.queue_size,
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

    let stats;
    let start = Instant::now();

    // ------ run the verifier ------
    if args.no_parallell {
        stats = run_sequential(dirverify, args.hash_file, sha_files);
    } else {
        stats = run_parallell(dirverify, args.hash_file, sha_files);
    }
    // ------

    if !args.no_summary {
        let seconds = start.elapsed().as_secs();
        println!("Summary:");
        println!("* Execution time: {}s", seconds);
        println!("* Read (files): {}", stats.read_files);
        println!("* Read (bytes): {}", stats.read_bytes);
        println!("* Bandwidth: {}", bandwidth(stats.read_bytes, seconds));
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

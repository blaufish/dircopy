use std::fs::File;
use std::io::Write;

use aes::Aes128;
use aes::cipher::BlockEncrypt;
use aes::cipher::KeyInit;
use aes::cipher::generic_array::{typenum::U16, GenericArray};
use clap::Parser;
//use serde::{Deserialize,Serialize};
use sha2::{Sha256, Digest};

fn fill_sha256_array(h1: &mut Sha256, length: usize) -> Box<[u8]> {
    let mut vec : Vec<u8> = Vec::with_capacity(length);
    let mut len = length;
    while len > 0 {
        let s = h1.finalize_reset();
        let slen = s.len();
        if len < slen {
            let short = &s[0..len];
            vec.extend_from_slice(short);
            break;
        }
        vec.extend_from_slice(&s);
        len = len - slen;
        h1.update(&s);
    }
    return vec.into_boxed_slice()
}

fn fill_sha256(mut file: File, seed: &[u8], length: usize) -> std::io::Result<()> {
    let mut h1 = Sha256::new();
    h1.update(seed);

    let mut len = length;
    let blocksize : usize = 32 * 1024;

    while len > 0 {
        if len <= blocksize {
            let array = fill_sha256_array(&mut h1, len);
            let fw = file.write_all(&array);
            return fw;
        }
        len = len - blocksize;
        let array = fill_sha256_array(&mut h1, blocksize);
        let fw = file.write_all(&array);
        let err = match fw {
            Err(_) => true,
            Ok(_) => false,
        };
        if err {
            return fw;
        }
    }
    Ok(())
}

fn aesctr_set(block: &mut GenericArray<u8, U16>, ctr: &mut GenericArray<u8, U16>) {
    for i in 0..15 {
        block[i] = ctr[i];
    }
    for i in 0..15 {
        if ctr[i] < 255 {
            ctr[i] = ctr[i] + 1;
            break;
        }
        ctr[i] = 0;
    }
}

fn fill_aesctr(mut file: File, seed: &[u8], length: usize) -> std::io::Result<()> {
    const BLOCKS : usize = 100;

    let mut h1 = Sha256::new();
    h1.update(seed);
    let mut s = h1.finalize_reset();

    let (left, right) = s.split_at_mut(16);

    let key : GenericArray<u8, U16> = GenericArray::clone_from_slice( left );
    let mut ctr : GenericArray<u8, U16> = GenericArray::clone_from_slice( right );

    let cipher = Aes128::new(&key);

    let empty = GenericArray::from([0u8; 16]);
    let mut len = length;
    let mut blocks = [empty; BLOCKS];

    let mut vec : Vec<u8> = Vec::with_capacity(16 * BLOCKS);
    while len > 0 {
        for mut block in blocks.iter_mut() {
            aesctr_set(&mut block, &mut ctr);
        }
        cipher.encrypt_blocks(&mut blocks);
        for block in blocks.iter_mut() {
            if len >= 16 {
                vec.extend_from_slice(block);
                len = len - 16;
            }
            else {
                let short = &block[0..len];
                vec.extend_from_slice(short);
                len = 0;
                break;
            }
        }
        let array = vec.clone().into_boxed_slice();
        let fw = file.write_all(&array);
        let err = match fw {
            Err(_) => true,
            Ok(_) => false,
        };
        if err {
            return fw;
        }
        vec.clear();
    }
    Ok(())
}

#[derive(
    clap::ValueEnum, Clone, Debug, serde::Serialize, serde::Deserialize
)]
#[derive(PartialEq)]
#[serde(rename_all = "kebab-case")]
enum Mode {
    AesCtr,
    Sha256
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

    #[arg(short, long)]
    mode: Mode,
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

    if args.mode == Mode::Sha256 {
        let _ = match fill_sha256(file, seed, length) {
            Err(why) => panic!("Error writing {}: {}", display, why),
            Ok(_) => ()
        };
    }
    else if args.mode == Mode::AesCtr {
        let _ = match fill_aesctr(file, seed, length) {
            Err(why) => panic!("Error writing {}: {}", display, why),
            Ok(_) => ()
        };
    }

    debug_assert!(false, "Debug build. This will cause extremely slow performance!");
}

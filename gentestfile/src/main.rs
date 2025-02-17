use std::fs::File;
//use std::io::prelude::*;
use std::io::Write;
use std::path::Path;
use sha2::{Sha256, Digest};


fn fill(mut file: File, seed: &[u8], length: usize) -> std::io::Result<()> {
    let mut len = length;
    let mut first = true;

    let mut h1 = Sha256::new();
    h1.update(seed);
    let mut s = h1.finalize_reset();

    while len > 0 {
        if first {
            first = false;
        }
        else {
            h1.update(&s);
            s = h1.finalize_reset();
        }

        let slen = s.len();
        if len >= slen {
            let fw = file.write_all(&s);
            if fw.is_err() {
                return fw;
            }
            len = len - slen;
        }
        else {
            //last block uneven
            let short = &s[0..len];
            let fw = file.write_all(short);
            return fw;
        }
    }
    Ok(())
}

fn main() {
    let path = Path::new("testfile.bin");
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    let seed = b"hello world";
    let _ = match fill(file, seed, 127) {
        Err(why) => panic!("Error writing {}: {}", display, why),
        Ok(_) => ()
    };
}

use hex_literal::hex;
use sha2::{Sha256, Digest};

fn main() {
    println!("Hello, world!");

    let mut hasher = Sha256::new();
    hasher.update(b"hello world");
    let result = hasher.finalize();
    let result_string = hex::encode(result);
    println!("{}", result_string);
    assert_eq!(result[..],
               hex!("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9")[..]
               );
}

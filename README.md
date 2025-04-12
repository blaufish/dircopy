# Rust Playground

I'm just having fun learning rust, nothing serious here so far!

Beware: everything here is super duper ultra beta alpha quality.

## Directory Copy with SHA256

Copies files fast.
Generates sha256sum files.
Multi-threaded.

Files:
* [dircopy/README.md](dircopy/README.md)
* [dircopy/src/main.rs](dircopy/src/main.rs)

## File Copy

Just testing various ways to copy files and benchmarking if anything matters.

* [filecopy/src/main.rs](filecopy/src/main.rs)
* [test.filecopy.sh](test.filecopy.sh)

System copy is a bit faster than copying through rust, but not that much.

``` plain
Cleaned files
+ ./filecopy/target/release/filecopy -i out/gen/sha/foo.1G.bin -o out/copy/foo.1G.bin.sha256mt -m sha256mt
Error: receiving on a closed channel
SHA: D87E1A61824F2C662FD882EA46771FFFCAE1550991F3E1A4D20F0D3853B1A902
real    0m1.373s
user    0m0.632s
sys     0m1.638s
+ ./filecopy/target/release/filecopy -i out/gen/sha/foo.1G.bin -o out/copy/foo.1G.bin.basic -m basic

real    0m1.093s
user    0m0.000s
sys     0m1.093s
+ ./filecopy/target/release/filecopy -i out/gen/sha/foo.1G.bin -o out/copy/foo.1G.bin.own -m own

real    0m1.148s
user    0m0.010s
sys     0m1.138s
+ ./filecopy/target/release/filecopy -i out/gen/sha/foo.1G.bin -o out/copy/foo.1G.bin.sha256 -m sha256
SHA: D87E1A61824F2C662FD882EA46771FFFCAE1550991F3E1A4D20F0D3853B1A902
real    0m1.656s
user    0m0.512s
sys     0m1.144s
+ cp -- out/gen/sha/foo.1G.bin out/copy/foo.1G.bin.system

real    0m1.077s
user    0m0.000s
sys     0m1.062s
d87e1a61824f2c662fd882ea46771fffcae1550991f3e1a4d20f0d3853b1a902  out/gen/sha/foo.1G.bin
d87e1a61824f2c662fd882ea46771fffcae1550991f3e1a4d20f0d3853b1a902  out/copy/foo.1G.bin.basic
d87e1a61824f2c662fd882ea46771fffcae1550991f3e1a4d20f0d3853b1a902  out/copy/foo.1G.bin.own
d87e1a61824f2c662fd882ea46771fffcae1550991f3e1a4d20f0d3853b1a902  out/copy/foo.1G.bin.sha256
d87e1a61824f2c662fd882ea46771fffcae1550991f3e1a4d20f0d3853b1a902  out/copy/foo.1G.bin.sha256mt
d87e1a61824f2c662fd882ea46771fffcae1550991f3e1a4d20f0d3853b1a902  out/copy/foo.1G.bin.system
```

## Generate Test Files

Generate test files.

[gentestfile/src/main.rs](gentestfile/src/main.rs)

Usage:
* `./gentestfile --help`
* `./gentestfile -o file --mode sha256 --length 512`
* `./gentestfile -o file --mode aes-ctr --length 512`

``` plain
Usage: gentestfile --output <OUTPUT> --seed <SEED> --length <LENGTH> --mode <MODE>

Options:
  -o, --output <OUTPUT>
  -s, --seed <SEED>
  -l, --length <LENGTH>
  -m, --mode <MODE>      [possible values: aes-ctr, sha256]
  -h, --help             Print help
  -V, --version          Print version
```

Length reflects the number of bytes (not bits...) of output generated.

SHA256 algorithm pseduo-code:

``` python
s = SHA256( seed )

while True:
  s = SHA256( s );
  copy(s, out);
```

AES-CTR algorithm pseudo-code:

``` python
key, ctr = SHA256( seed )

while True:
  s = AES128_Encrypt(key, ctr)
  ctr = ctr + 1
  copy(s, out);
}
```

_AES is potentially faster due to CPU optimizations._

## Hello World

A very simple test application...

[hello\_world/src/main.rs](hello_world/src/main.rs)


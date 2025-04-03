# Rust Playground

I'm just having fun learning rust, nothing serious here so far!

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


# dirverify: directory verifier using SHA256 files

`dirverify dir` searches a directoy for `shasum*.txt`
  files and then verifies contents using `SHA256`.
File format compatible with `dircopy` and `sha256sum`.

`dirverify dir1 dir2 dir3...` searches multiple
  directories for `shasum*.txt` files, and verifies
  the directories.

`dirverify --hash-file shasum.txt dir1 dir2 dir3...`
  verifies multiple directories against a single `shasum.txt` file.

## Usage

`dirverify -h`

``` plain
A directory verifier. Searches for shasum*.txt files in directories

Usage: dirverify [OPTIONS] [DIR]...

Arguments:
  [DIR]...  Directories with files to be verified

Options:
      --hash-file <HASH_FILE>    Specify sha256-file, and disable automatic search for shasum*.txt files
      --silent                   Inhibit all stdout print outs
      --no-convert-paths         Keep paths exactly as is. Do not try to workaround unix, dos mismatches
      --no-summary               Do not print a summary
      --no-threaded-sha          Disable threaded sha read/hash behavior
      --no-parallell             Do not check multiple directories at the same time
      --queue-size <QUEUE_SIZE>  Size of queue between reader and hasher thread. Tuning parameter [default: 2]
      --block-size <BLOCK_SIZE>  Size of blocks between reader and hasher thread. Tuning parameter [default: 128K]
      --verbose                  Print informative messages helpful for understanding processing
  -h, --help                     Print help
  -V, --version                  Print version
```

## Behaivor modifiers

`--hash-file <HASH_FILE>` disables detecting `shasum*.txt` files, and instead
  uses the file specified.

`--no-convert-paths` disables attempts to automatically resolve file/path
  interoperability issues between Linux / UNIX / Windows / DOS.
  Overrides the helpful slash-fixing default;
  * `\` paths will be converted into `/` on Linux, Unix.
  *  `/` paths will be converted into `\` on Windows, DOS.

## Printing more or less information

`--no-summary` will inhibit summary message like this:

``` plain
Summary:
* Execution time: 805s
* Read (files): 404
* Read (bytes): 2107210129938
* Bandwidth: 2.618 GB/s
* Files matching: 399
* Files mismatching: 5
* Errors: 48
```

`--silent` inhibit all print outs except error (`stderr`) messages.

`--verbose` enables additional messages, such as printing out which
  `shasum*.txt` files are detected and used.

Impossible combinations will result in error.
  You cannot be both `--verbose` and `--silent`.

## Concurrency

Default behaivor is to use threading where possible to increase performance.

`--no-parallell` will inhibit running `dirverify` on multiple directories
  in parallell, and forcing sequential execution.
  Should be significantly slower.

`--no-threaded-sha` will inhibit having one reader-thread and one
  hasher-thread, forcing sequential execution.
  Appears to be about 10% slower.

## Tuning parameters

`--queue-size <QUEUE_SIZE>` affects how many blocks may queued.

`--block-size <BLOCK_SIZE>` affects how large blocks can be read from source.

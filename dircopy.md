# dircopy: directory copy with SHA256 file genator


## Usage

`./dircopy/target/release/dircopy -h`

``` plain
Usage: dircopy [OPTIONS] --input <INPUT> --output <OUTPUT>

Options:
  -i, --input <INPUT>
  -o, --output <OUTPUT>
      --queue-size <QUEUE_SIZE>              [default: 10]
      --block-size <BLOCK_SIZE>              [default: 128K]
      --overwrite-policy <OVERWRITE_POLICY>  [default: default]
  -h, --help                                 Print help
  -V, --version                              Print version
```

## Performance Tuning

Defaults provided, block size `128K` and queue size `10` appears
great when testing on my machine.

WSL hurts Windows performance significantly, avoid.

See [performance.md](performance.md) for more details.

Windows example; copying `2.4 TiB` to a `271 MBps` destination disk
in `2 hrs 44 min`, reaching `266.553 MB/s`
(`98.4%` of maximal theoretical utilization):

`dircopy-v0.0.0-37-g6144bc7-dirty-x86_64-pc-windows-gnu.exe -i PATH_NAS -o PATH_HDD`

``` plain
Block size: 131072
Queue size: 10
Overwite policy: default
Writing SHA256 sums to: PATH_HDD\shasum.2025-04-12.17.39.35.txt
2TiB 389GiB 550MiB | 266.553 MB/s | 1384 files
Execution time: 9819s
Average bandwidth: 266.553 MB/s
```

## Dangerous parameters

`--overwrite-policy <OVERWRITE_POLICY>` affects how likely the tool
is to overwrite existing files.

* `default` does a best effort in attempting to avoid accidental
  overwrites.
  It does perform sanity checks on file length etc.
* `never` is safest mode.
  Files in output directory will never be overwritten.
  It is 100% impossible to overwrite files with this strategy...
* `always` is **danger mode**.
  Files will always be overwritten.
  Probably only useful for benchmarking tool etc.

## Thread design

`main` thread:
* command and controls
* receiving statistic updates from `router_thread`.
* normal non-error screen output.
* writing `sha256sum.txt` files to disk.

Additional threads are:

`read_thread`:
* reads from disk
* data onto queues.

`router_thread` puts data onto queues for downstream threads:
* `sha_thread`
* `file_write_thread`
* `main`

`sha_thread`: calculates `SHA256`.

`file_write_thread` writes data to destination.

Between each thread there are up to `<QUEUE_SIZE>`
blocks in buffers to reduce chance of unecessary stalls
in the copy/hash pipeline :-)


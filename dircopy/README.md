# Directory Copy with SHA256

Tool for copying large sets of media files, e.g. terabytes.
Generates SHA256 sum files,
enabling verification that data was transmitted successfully.

Files:
* [src/main.rs](src/main.rs)
* [build-with-podman.sh](build-with-podman.sh)
  * builds project using Dockerfile/Podman.
  * exports standard Linux/WSL binary.
  * exports Windows cross compiled binary.
* [build-test.sh](build-test.sh)
  generates random files,
  copies files,
  verifies source and destination match sha256sums.
* [Dockerfile](Dockerfile) - building froms scratch

Usage:

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

## Performance Tuning

### Linux Subsystem for Windows (WSL) considerations

WSL hurts Windows performance significantly.

Testing same scenarios in WSL appears to take **3 - 4 times**
longer to execute, compared to Windows native.

If performance is of the essence, avoid WSL.

### Defaults provided

Defaults provided, block size `128K` and queue size `10` appears
great when testing on my machine.

_Example performance observed with these defaults:_

**Toshiba MG10AFA22TE Series SATA HDD 271 MBps (512 MB cache)**
* `266.553 MB/s` observed when copying from 10GbE SSD RAID NAS
  over USB 3.1 (Gen 2) USB-C 10Gbit/s RaidSonic
  ICY BOX IB-377-C31 enclosure.
  I.e. destination drive can reach `98.4%` of theoretical max
  utilization if source drive is very fast.
* `195.027 MB/s` observed when copying from internal SSD
  (Samsung EVO 870 EVO 4TB SATA) to HDD,
  over an USB 3.1 (Gen 2) USB-C 10Gbit/s RaidSonic
  ICY BOX IB-377-C31 enclosure.
* `189.559 MB/s` observed when copying from internal SSD
  (Samsung EVO 870 EVO 4TB SATA) to HDD,
  over an old USB -> SATA adapter
  (identifying itself as SCSI disk device).

**Samsung SSD EVO 970 Plus 1TB NVME**
* `965.849 MB/s` observed when copying between directories on same disk.
* `1.844 GB/s` occassionally observed...? (windows caching read-side, maybe).

**Samsung EVO 870 EVO 4TB SATA**
* `471.692 MB/s` observed when copying between directories on same disk.

### Queue size

`--queue-size <QUEUE_SIZE>` controls how many blocks
can be queued up between threads.

A small number should suffice.
`10` (default) appears generally good.

Changing values does **not** appear to have meaningful impmact.
* Minimal value `1` only appears to have marginal speed degradations,
  if any.
* `1000` does not provide any observable performance boost.

### Block size

`--block-size <BLOCK_SIZE>` controls size of blocks, i.e. size of
disk read, writes.

`128K` (Default) appears close to optimal.

* A few megabytes, e.g. `1M` to `8M` should suffice for most users.
* Too small values, such as `1K`, seems to hurt performance.
* Too large values, such as `1G`, hurts performance significantly.

Resonable values appears optimal for keeping source & destination
working well.

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

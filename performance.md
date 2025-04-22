# Performance tuning

## Linux Subsystem for Windows (WSL) considerations

WSL hurts Windows performance significantly.

Testing same scenarios in WSL appears to take **3 - 4 times**
longer to execute, compared to Windows native.

If performance is of the essence, avoid WSL.

## Performance with provided defaults

Defaults provided, block size `128K` and queue size `10` appears
great when testing on my machine.

### External disk-to-disk tests

Copies from one drive to another:

| Source               | Destination                              | Performance          |
| -------------------- | ---------------------------------------- | :------------------- |
| 10GbE SSD RAID NAS   | Toshiba MG10AFA22TE over IB-377-C31      | 266.553 MB/s (98.4%) |
| Samsung EVO 870 SATA | Toshiba MG10AFA22TE over IB-377-C31      | 195.027 MB/s (72%)   |
| Samsung EVO 870 SATA | Toshiba MG10AFA22TE over old USB adapter | 189.559 MB/s (70%)   |

### Local tests

Copying files within a drive:

| Drive                             | Performance                     |
| --------------------------------- | :------------------------------ |
| Samsung EVO 970 Plus 1TB SSD NVME | 965.849 MB/s (up to 1.844 GB/s) |
| Samsung EVO 870 EVO 4TB SSD SATA  | 235.847 MB/s (up to 482.9 MB/s) |

Notably the outliers with extreme performance should be ignored :)

### Benchmarking is hard

Extreme speed-ups observed on re-running internal file copy tests,
that simply do not make sense.

Caching is interfering with benchmarks if benchmarking with:
* 64GiB of system RAM,
* 18GiB of test files,
  "impossible" performance is observed.
* 147GiB of test files,
  "impossible" performance is no longer observed.
* i.e. benchmarks can yield impossible results when file sizes are
  small enough for test files to be cached in computer RAM...

Example: SATA-II is a 600 MB/s;
* 235.847 MB/s makes sense for SSD read/write.
  `2 * 235.847 = 471.7` or **79%** of theoretical max.
* _For reference, Windows own file copy dialog average 200 - 220 MB/s
  when copying large files...  which makes sense, _
  `210*2/600 = 70%` _is decent!_
* 482.9 MB/s read and write makes no sense.
  `482.9*2/600 = 161%` ...
  Clearly **161%** performance is impossible,
  RAM caching issue.
* Impossible performance not reproducible when file sizes are
  significantly larger than system RAM.

### Additional details

Drives used in test and additional details;

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
* `266.553 MB/s` observed when copying files.
  This makes sense as reading and writing at this speed would be `533 MB/s`
  or **89%** of SATA-II theoretical max of 600 MB/s.
* `471.692 MB/s`, `482.924 MB/s`
  observed when copying between directories on same disk.
  These values are nonsensical, Windows caching read-side maybe?

## Queue size

`--queue-size <QUEUE_SIZE>` controls how many blocks
can be queued up between threads.

A small number should suffice.
`10` (default) appears generally good.

Changing values does **not** appear to have meaningful impmact.
* Minimal value `1` only appears to have marginal speed degradations,
  if any.
* `1000` does not provide any observable performance boost.

## Block size

`--block-size <BLOCK_SIZE>` controls size of blocks, i.e. size of
disk read, writes.

`128K` (Default) appears close to optimal.

* Small buffers, e.g. `128K`, `1M` to `8M` should suffice for most users.
* Too small values, such as `1K`, seems to hurt performance.
* Too large values, such as `1G`, hurts performance significantly.

Resonable values appears optimal for keeping source & destination
working well. HDD sound less when operating, and succeeds faster.

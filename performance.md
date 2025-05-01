# Performance tuning

All performance tests are performed with few very large files,
i.e. simulating transfer of large media files.

## Linux Subsystem for Windows (WSL) considerations

WSL hurts Windows performance significantly.

Testing same scenarios in WSL appears to take **3 - 4 times**
longer to execute, compared to Windows native.

If performance is of the essence, avoid WSL.

## Native Windows performance

"_On my machine_" **(TM)**
tool appears to perform on par with Robocopy and Windows file copy
dialogs, or a slight percentage faster.

## Performance with provided defaults

Defaults provided, block size `128K` and queue size `10` appears
great when testing on my machine.

### External disk-to-disk tests

Copies from one drive to another:

| Source                 | Destination                              | Performance  | Notes     |
| ---------------------- | ---------------------------------------- | :----------- | --------- |
| 10GbE SSD RAID NAS     | Toshiba MG10AFA22TE over IB-377-C31      | 266.553 MB/s | 98.4% `1` |
| Samsung EVO 870 SATA   | Toshiba MG10AFA22TE over IB-377-C31      | 195.027 MB/s | 72% `1`   |
| Samsung EVO 870 SATA   | Toshiba MG10AFA22TE over old USB adapter | 189.559 MB/s | 70% `1`   |
| Kingston SDR2V6/256GB  | Samsung EVO 870 EVO 4TB SSD SATA         | 295.218 MB/s | 105% `2`  |
| Samsung EVO 870 SATA   | Kingston SDR2V6/256GB                    | 194.394 MB/s | 130% `3`  |

Notes:
* Listed performance is from very large test cases,
  other other messures, to ensure caching does impact test too much.
  For SD cards, tested with copying full card.
* `1`: Performance compared to MG10AFA22TE rated max of 271 MBps.
* `2`: Performance compared to Kingston SDR2V6/256GB advertised, rated max of 280 MB/s read.
* `3`: Performance compared to Kingston SDR2V6/256GB advertised, rated max of 150 MB/s write.

### Local tests

Copying files within a drive:

| Drive                             | Performance  | Outliers (cached reads)  |
| --------------------------------- | :----------- | :----------------------- |
| Samsung EVO 970 Plus 1TB SSD NVME | 965.849 MB/s | up to 1.8 GB/s           |
| Samsung EVO 870 EVO 4TB SSD SATA  | 235.847 MB/s | up to 482.9 MB/s         |

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

**Kingston SDR2V6/256GB UHS-II Canvas React Plus 256GB up to 280MB/s**
* Listed card performance:
  Class 10, UHS-II, U3, V60.
  280/150MB/s read/write (256GB-1TB).
* `295.218 MB/s` (105%) read out of rated `280 MB/s` observed
  with **Kingston WFS-SD** UHS-II reader.
* `194.394 MB/s` (130%) write out of rated `150MB/s` observed
  with **Kingston WFS-SD** UHS-II reader.
* Under promise and over deliver; nice Kingston, nice.
  Selling 295 MB/s card (281 MiB/s) but putting 280 MB/s on
  packaging?
  "_235GiB 78MiB 512KiB | 295.218 MB/s | 258 files_" (252GB),
  "_Execution time: 855s_".
  Seems be approximately **281.5 MiB/s**.
* USB-cable super critical with **WFS-SD** card reader.
  Reader degraded to `30 MB/s` on an USB 3 type A cable.
  Which is confusing as higher speeds previously observed
  with other cards...
  Check USB-port, USB-cable, reader if this card isn't
  performing, as wrong cable/port has insane impact with
  this card?...

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

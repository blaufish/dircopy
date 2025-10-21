# Directory Copy and Verification with SHA256

Tools for copying large sets of media files, e.g. terabytes.
Generates SHA256 sum files,
  enabling verification that data was transmitted successfully.
Utilizes threading where possible to mimimize unecessary waits;
  improving performance and reducing wall clock time.

Usage:
* `dircopy [OPTIONS] --input <INPUT> --output <OUTPUT>`
* `dirverify [OPTIONS] [DIR]...`

`dircopy` files:
* [dircopy.md](dircopy.md) - manual
* [src/bin/dircopy.rs](src/bin/dircopy.rs)

`dirverify` files:
* [dirverify.md](dirverify.md) - manual
* [src/bin/dirverify.rs](src/bin/dirverify.rs)

Performance guidance, tests:
* [performance.md](performance.md)

Changelog
* [changelog.md](changelog.md)

Auxiliary files:
* [build-with-podman.sh](build-with-podman.sh)
  * builds project using Dockerfile/Podman.
  * exports standard Linux/WSL binary.
  * exports Windows cross compiled binary.
* [build-test.sh](build-test.sh)
  generates random files,
  copies files,
  verifies source and destination match sha256sums.
* [Dockerfile](Dockerfile)
* [tag-and-changelog.sh](tag-and-changelog.sh)
  tag releases.

## Historical note

`NOTE`:
`dircopy` is forked from [blaufish/rust-playground](https://github.com/blaufish/rust-playground)
where I practice learning Rust.

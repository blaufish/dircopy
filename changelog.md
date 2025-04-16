# 0.1.1

> Release version 0.1.1. First release tag.

Files:
``` plain
A	.dockerignore
M	.gitignore
R060	gentestfile/Cargo.lock	Cargo.lock
R086	dircopy/Cargo.toml	Cargo.toml
A	Dockerfile
M	README.md
A	build-test.sh
A	build-with-podman.sh
D	dircopy/Cargo.lock
D	dircopy/src/main.rs
D	filecopy/Cargo.lock
D	filecopy/Cargo.toml
D	filecopy/src/main.rs
D	gentestfile/Cargo.toml
D	gentestfile/src/main.rs
D	hello_world/Cargo.lock
D	hello_world/Cargo.toml
D	hello_world/src/main.rs
D	init.sh
A	performance.md
A	src/main.rs
A	tag-and-changelog.sh
D	test.filecopy.sh
D	test.gen.sh
D	test.gen.sha256.txt
```

Commits:
``` plain
* b4d5529 Tag releases
* ba685b6 Restructure directory
* a497460 performance.md
* 24695ae Add subdirs to tests
* 8e516bf Add self-test to builds
* f1daf92 Clarify test cases
* f650f7a rustfmt
* a15f01d Compact Dockerfile
* bcb118b Document testing with new HDD enclosure
* 6144bc7 128K Default optimal
* f52ba51 Print speed in regular debug messages
* 8518a38 Print out bandwidth
* 67f6420 Docker build files
* cf5c20a Document latest dircopy version
* 9e739c9 Implement overwrite policies
* 365f187 block size from cli arguments
* 4f66e9a Signal downstream threads of read failure
* 0f411c3 replace hash sync_channel with thread join value
* 8725044 Make more use of ?
* 3d91c44 Simplify read thread
* 8b0037e Move debug messaging into Configuration class
* 265956c remove some warnings
* 8b68cd8 Document dircopy
* 087ee43 Emit occassional debug messages
* bb28a53 copy_directory wrapper
* 265682e generate better sha256sum formatting
* c4f602a Make copy return hash of file
* 7d7415b De-complex message flow
* 7b4221e break loop when done
* ce6a1b0 copy kind of
```

# v0.1.0

> dircopy initial tag
> 

Commits:
``` plain
* cae2b30 dircopy early draft
* 4c756b5 init.sh refactor
* c6b1a1b Multithreaded, improved
* a33b496 Mt test
* 03aac21 print out hash
* 9586dd3 sha256 variant
* 17e848a File copy with loop
* 3a49bd5 filecopy
* c0ddff5 README
* e726483 Implemented AES-CTR option
* 0437e6d Limit memory usage to 32 * 1024
* 5cd1457 Test case
* 54d7017 Warn on debug builds
* a2ad6a3 Simplify loop
* 91eac20 Still hilariously slow but whatever
* 1c77831 This is slow as F, but clearer?
* 54c38ac Command line parsing
* f456329 Remove redundant first hash
* 9afa400 Fix off-by-1 bug :)
* 1cb9281 Gentestfile
* 19869b7 Sha256 hello world
* 9e8ffe2 Hello World
```

FROM ubuntu:noble

RUN \
 apt update -y && \
 apt install -y \
   binutils-mingw-w64-x86-64 \
   build-essential \
   llvm \
   mingw-w64 \
   rustup

RUN \
 rustup default stable && \
 rustup target add x86_64-pc-windows-gnu


COPY \
 Cargo.lock \
 Cargo.toml \
 /build/

RUN \
 cd /build/ && \
 mkdir -v src && \
 echo 'fn main() {}' > src/main.rs && \
 cargo fetch && \
 cargo check && \
 rm -Rvf -- src

COPY src/bin/*.rs /build/src/bin/

RUN cd /build/ && cargo build --release
RUN cd /build/ && cargo build --release --target x86_64-pc-windows-gnu

COPY \
 build-test.sh \
 /build/

RUN cd /build && \
 ./build-test.sh && \
 rm -rf -- ".test"

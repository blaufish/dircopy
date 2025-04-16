#!/bin/bash

set -e
set -x

if [[ "$DIR" == "" ]]
then
	DIR="$PWD/.test"
fi

mkdir -p -- "$DIR/src/"
touch -- "$DIR/src/0"
for size in 1 1022 1023 1024 1025 1026 1048573 1048574 1048576 1048577 1048578
do
	dd if=/dev/urandom of="$DIR/src/$size" count=1 bs="$size"
done

mkdir -p -- "$DIR/dst/"

target/release/dircopy -i "$DIR/src" -o "$DIR/dst" --overwrite-policy always

SHASUM=$( find "$DIR/dst" -name "shasum.*.txt" | sort --version-sort | tail -1 )

cd -- "$DIR/src/" && sha256sum -c -- "$SHASUM" && cd ..
cd -- "$DIR/dst/" && sha256sum -c -- "$SHASUM" && cd ..

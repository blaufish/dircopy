#!/bin/bash

set -e
set -x

DIR="$PWD/.test"
CUR="$PWD"

while getopts ":d:c:" option; do
	case "${option}" in
		d)
			DIR="${OPTARG}"
			;;
		c)
			CUR="${OPTARG}"
			;;
		*)
			exit 1
			;;
	esac
done

mkdir -p -- "$DIR/src/"
mkdir -p -- "$DIR/src/subdir_a/subdir_b"
mkdir -p -- "$DIR/src/subdir_c"
for subdir in "$DIR/src/" "$DIR/src/subdir_a/subdir_b" "$DIR/src/subdir_c"
do
	touch -- "$subdir/0"
	for size in 1 1022 1023 1024 1025 1026 1048573 1048574 1048576 1048577 1048578
	do
		dd if=/dev/urandom of="$subdir/$size" count=1 bs="$size"
	done
done

mkdir -p -- "$DIR/dst/"
find "$DIR/dst" -name "shasum.*.txt" -exec rm -- '{}' ';'

target/release/dircopy -i "$DIR/src" -o "$DIR/dst" --overwrite-policy always

SHASUM=$( find "$DIR/dst" -name "shasum.*.txt" | sort --version-sort | tail -1 )

cd -- "$DIR/src/" && sha256sum -c -- "$SHASUM"
cd -- "$DIR/dst/" && sha256sum -c -- "$SHASUM"

cd -- "$CUR"
target/release/dirverify --verbose "$DIR/dst"
target/release/dirverify --verbose --hash-file "$SHASUM" "$DIR/src" "$DIR/dst"
target/release/dirverify --verbose --no-parallell "$DIR/dst"
target/release/dirverify --verbose --no-parallell --hash-file "$SHASUM" "$DIR/src" "$DIR/dst"
target/release/dirverify --verbose --no-threaded-sha "$DIR/dst"
target/release/dirverify --verbose --no-threaded-sha --hash-file "$SHASUM" "$DIR/src" "$DIR/dst"

#!/bin/bash

copy() {
	local f_in="$1"
	local f_out="$2"
	local mode="$3"
	local bin="./filecopy/target/release/filecopy"

	if [[ "$DEBUG" == "Y" ]]
	then
		bin="./filecopy/target/debug/filecopy"
	fi

	case "$mode" in
		system)
			(set -x; time cp -- "$f_in" "$f_out")
			;;
		*)
			(set -x; time "$bin" -i "$f_in" -o "$f_out" -m "$mode")
			;;
	esac
}

set -e
if [[ "$DEBUG" == "Y" ]]
then
	cd filecopy && cargo build && cd ..
else
	cd filecopy && cargo build --release && cd ..
fi

mkdir -p -- out/copy

copy "out/gen/sha/foo.1G.bin" "out/foo.1G.bin.basic" basic
copy "out/gen/sha/foo.1G.bin" "out/foo.1G.bin.system" system

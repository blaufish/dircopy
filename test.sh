#!/bin/bash

gen() {
	local file="$1"
	local mode="$2"
	local seed="$3"
	local length="$4"
	local bin="./gentestfile/target/release/gentestfile"

	if [[ "$DEBUG" == "Y" ]]
	then
		bin="./gentestfile/target/debug/gentestfile"
	fi

	echo "Generating $file, $length bytes, mode: $mode, seed $seed"
	if [[ "$TIME" == "Y" ]]
	then
		time \
		RUST_BACKTRACE=1 \
		"$bin" \
		--output "$file" \
		--seed "$seed" \
		--length "$length" \
		--mode "$mode"
	else
		RUST_BACKTRACE=1 \
		"$bin" \
		--output "$file" \
		--seed "$seed" \
		--length "$length" \
		--mode "$mode"
	fi
}

set -e
#set -x
if [[ "$DEBUG" == "Y" ]]
then
	cd gentestfile && cargo build && cd ..
else
	cd gentestfile && cargo build --release && cd ..
fi

mkdir -p -- out/gen/sha
mkdir -p -- out/gen/aes

for size in {0..127}
do
	gen "out/gen/sha/test.$size.bin" "sha256" "test" "$size"
	gen "out/gen/aes/test.$size.bin" "aes-ctr" "test" "$size"
	gen "out/gen/sha/word.$size.bin" "sha256" "word" "$size"
	gen "out/gen/aes/word.$size.bin" "aes-ctr" "word" "$size"
done

gen "out/gen/sha/foo.1G.bin" "sha256" "foo" 1073741824
gen "out/gen/aes/foo.1G.bin" "aes-ctr" "foo" 1073741824

#sha256sum -c "test.gen.sha256.txt"

#gen "out/gen/world.G1.bin" "world" 137438953472 # 1 Gi Bit

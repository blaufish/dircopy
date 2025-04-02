#!/bin/bash

gen() {
	local file="$1"
	local seed="$2"
	local length="$3"
	echo "Generating $file, $length bytes, seed $seed"
	if [[ "$TIME" == "" ]]
	then
		./gentestfile/target/release/gentestfile \
		--output "$file" \
		--seed "$seed" \
		--length "$length"
	else
		time \
		./gentestfile/target/release/gentestfile \
		--output "$file" \
		--seed "$seed" \
		--length "$length"
	fi
}


set -e
cd gentestfile && cargo build --release && cd ..

mkdir -p -- out/gen
gen "out/gen/test.8.bin" "test" 1
gen "out/gen/test.248.bin" "test" 31
gen "out/gen/test.256.bin" "test" 32
gen "out/gen/test.264.bin" "test" 33
gen "out/gen/test.504.bin" "test" 63
gen "out/gen/test.512.bin" "test" 64
gen "out/gen/test.520.bin" "test" 65
gen "out/gen/hello.256.bin" "hello" 32
gen "out/gen/world.256.bin" "world" 32
gen "out/gen/world.512.bin" "world" 64
gen "out/gen/world.M1.bin" "world" 134217728 # 1 Mi Bit
gen "out/gen/world.M8.bin" "world" 1073741824 # 8 Mi Bit, 1 Mi Byte

sha256sum -c "test.gen.sha256.txt"

#gen "out/gen/world.G1.bin" "world" 137438953472 # 1 Gi Bit

#!/bin/bash

set -e
set -x

TAG="$(git describe --long --dirty)"
IMAGE=localhost/rust-playground-dircopy
IMAGE_TAG="$IMAGE:$TAG"
OUTDIR="$PWD/out"

podman_copy() {
	local src="$1"
	local dst="$2"
	podman run --rm -v "$OUTDIR":/out:rw "$IMAGE_TAG" \
		cp -- "$src" "/out/$dst"
	if [[ "$EXTERNAL_DIR" != "" ]]
	then
		cp -- "$OUTDIR/$dst" "$EXTERNAL_DIR/$dst"
	fi
}

podman build \
	-t "$IMAGE_TAG" \
	.

mkdir -p -- "$OUTDIR"
podman_copy /build/target/x86_64-pc-windows-gnu/release/dircopy.exe "dircopy-$TAG-x86_64-pc-windows-gnu.exe"
podman_copy /build/target/x86_64-pc-windows-gnu/release/dirverify.exe "dirverify-$TAG-x86_64-pc-windows-gnu.exe"
podman_copy /build/target/release/dircopy "dircopy-$TAG"
podman_copy /build/target/release/dirverify "dirverify-$TAG"

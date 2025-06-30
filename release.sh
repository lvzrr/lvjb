#!/bin/bash

read -rp "Squash binary size? [y/N] " answer3

export OPENSSL_DIR="/opt/openssl-musl"
FLAGS="-C link-args=-s -C panic=abort -C debuginfo=0 --remap-path-prefix=$HOME=. --remap-path-prefix=$(pwd)=."
BUILD_CMD="cargo build --release --target x86_64-unknown-linux-musl"
RUSTFLAGS="$FLAGS" eval "$BUILD_CMD"
cp ./target/x86_64-unknown-linux-musl/release/lvjb .

strip --strip-all ./target/x86_64-unknown-linux-musl/release/lvjb
objcopy --remove-section=.comment --remove-section=.note.gnu.build-id --strip-unneeded lvjb

if [[ "$answer3" =~ ^[Yy]$ ]]; then
	upx --ultra-brute lvjb
fi

cargo clean

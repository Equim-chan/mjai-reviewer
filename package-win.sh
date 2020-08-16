#!/usr/bin/env bash
# NOTE: This script is not intended for general uses.

echo 'NOTE: make sure to have removed `-march=native` and `-C target-cpu=native`.'
set -ex

AKOCHAN_DIR=$HOME/akochan
REVIEWER_DIR=$HOME/akochan-reviewer
OUT_DIR="$(mktemp -d)"

pushd "$AKOCHAN_DIR"
pushd ai_src
make -j$(nproc)
popd
make -j$(nproc)
mkdir -p "$OUT_DIR"/akochan
cp -rt "$OUT_DIR"/akochan \
    system.exe \
    ai.dll \
    params \
    README.md \
    LICENSE \
    /mingw64/bin/libgcc_s_seh-1.dll \
    /mingw64/bin/libgomp-1.dll \
    /mingw64/bin/libstdc++-6.dll \
    /mingw64/bin/libwinpthread-1.dll
popd

pushd "$REVIEWER_DIR"
cargo build --release
cp -rt "$OUT_DIR" \
    target/release/akochan-reviewer.exe \
    tactics.json \
    README.md \
    LICENSE
popd

7z a -tzip akochan-reviewer.zip "$OUT_DIR"/*
rm -rf "$OUT_DIR"

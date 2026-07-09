#!/usr/bin/env sh
set -eu

echo "cargo test --lib"
time cargo test --lib

echo "cargo build --release"
time cargo build --release

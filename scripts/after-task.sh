#!/usr/bin/env sh
set -eu

cargo clippy --all-targets -- -D warnings
cargo test
npm run build

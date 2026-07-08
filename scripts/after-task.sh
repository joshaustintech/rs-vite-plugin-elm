#!/usr/bin/env sh
set -eu

cargo test
npm run build

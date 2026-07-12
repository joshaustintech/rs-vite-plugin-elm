#!/usr/bin/env sh
set -eu

cargo clippy --all-targets -- -D warnings
npm test
npm run build
sh ./scripts/security-post-action.sh

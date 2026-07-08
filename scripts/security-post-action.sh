#!/usr/bin/env sh
set -eu

printf '%s\n' '[security-post-action] attacker-minded review'
printf '%s\n' '[security-post-action] only findings that satisfy AGENT_HARNESS.md count'

git diff --unified=0 -- . \
  | rg -n '^\+.*(Command::new|std::process::Command|unsafe|unwrap\(|expect\(|panic!|todo!|eval\(|innerHTML|document\.write|from_utf8_unchecked|set_var\(|read_to_string\(|write\(|remove_file\()' \
  || true

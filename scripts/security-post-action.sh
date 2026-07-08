#!/usr/bin/env sh
set -eu

printf '%s\n' '[security-post-action] attacker-minded review'
printf '%s\n' '[security-post-action] only findings that satisfy AGENT_HARNESS.md count'
printf '%s\n' '[security-post-action] review watchlist: scripts/security-watchlist.md'

git diff --unified=0 -- . \
  | rg -n '^\+.*(Command::new|std::process::Command|unsafe|unwrap\(|unwrap_unchecked|expect\(|panic!|todo!|eval\(|innerHTML|document\.write|from_utf8_unchecked|from_utf8_lossy|get_unchecked|set_var\(|env::var\(|thread::spawn|spawn\(|temp_dir\(|canonicalize\(|read_link\(|symlink|hard_link|rename\(|set_permissions\(|read_to_string\(|write\(|remove_file\()' \
  || true

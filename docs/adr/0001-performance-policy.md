# ADR 0001: Keep performance checks out of CI tests

Status: accepted

Problem:
- The repo had benchmark-like unit tests that asserted one code path was faster than another.
- That makes CI flaky because timing depends on host load, filesystem state, cache warmth, and runner noise.

Decision:
- Keep functional correctness in `cargo test`, `npm run build`, and `./scripts/after-task.sh`.
- Keep any profiling or timing experiments in a local-only harness.
- Do not assert elapsed time in CI/CD tests.

Consequences:
- Performance work becomes an explicit local activity instead of a pass/fail gate.
- CI stays deterministic and focused on behavior.
- If we need to compare paths, we do it with ad hoc timing commands or a dedicated local script, not unit-test assertions.

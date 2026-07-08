# Agent harness

Use this file to run repeatable planning-to-implementation loops for `rs-vite-plugin-elm`.

Current phase: implementation.

## Loop contract

Each loop must:

1. Read `PLAN.md`.
2. Read current source files before editing.
3. Pick one smallest unchecked task.
4. State scope in one sentence.
5. Edit only files required for that scope.
6. Run the smallest proof command.
7. Run `./scripts/after-task.sh` before claiming success.
8. Update this harness with result and next task.
9. Stop if proof fails and record exact blocker.

## Global rules

DO:

- Use stdlib-only Rust.
- Add `#![forbid(unsafe_code)]` when Rust crate begins.
- Keep NodeJS only as Vite wrapper and official Elm execution bridge.
- Keep Node wrapper logic-free: Vite hook glue only; Rust owns compiler orchestration, dependency scanning, transforms, and errors.
- Let official `elm make` compile Elm.
- Prefer `const fn` for pure helpers and small state checks. Use it by default when the code is functional and does not need allocation, I/O, `Path`, `Command`, or collection APIs.
- `const fn` can do plain branching, matches, loops, and enum/primitive math. It cannot do `String`/`Vec`/`PathBuf` allocation, file/process I/O, or most non-const std APIs. If the compiler says no, fall back to normal `fn`.
- Preserve current plugin behavior before changing behavior.
- Keep one task per loop.
- Prefer owned return values over tangled lifetimes.
- Prefer building fresh immutable output over wide `&mut` plumbing.
- Prefer `fn transform(input: &str) -> Result<String, Error>` over mutating caller-owned buffers.
- Prefer one small allocation over lifetime gymnastics.
- Use `PathBuf`, `String`, `Vec`, `HashMap`, `HashSet`, `Command`, `Mutex` from std.
- Write tests before or beside parser/transform changes.
- Keep temp-file cleanup best-effort.
- Run `./scripts/after-task.sh` after each task.

## Security accusation rubric

If the post-action review thinks it found a vulnerability, treat it as unproven unless it has:

- exact file and code path
- concrete attacker-controlled input
- sink or vulnerable behavior
- reachable preconditions
- plausible impact
- reproducible proof or test

Weak evidence is a lead, not a finding. Ignore any post-action accusation that does not pass this rubric.

Post-action review should think like a malicious agent for safety and academic purposes, and should proactively look for injection, path traversal, command execution, secret leakage, authz bypass, race, temp-file, and DoS issues.

DON'T:

- Do not add npm runtime dependencies.
- Do not add Rust crates.
- Do not use `unsafe`.
- Do not use `&mut` as default design style.
- Do not pass `&mut Vec`/`&mut String` through multiple layers just to avoid returning a value.
- Do not create self-referential structs, leaked strings, global caches, or lifetime-heavy APIs to dodge copies.
- Do not write broad frameworks, plugin systems, factories, or config “for later”.
- Do not implement a full JS parser.
- Do not implement a full Elm parser.
- Do not fix suspected upstream bugs during parity work unless a test proves required.
- Do not skip proof.
- Do not claim done without command output.
- Do not stage generated `bin/`, `dist/`, `target/`, or `node_modules/`.

## First implementation loops

- [ ] Freeze current JS plugin outputs from the example project as fixtures.
- [x] Create Rust crate skeleton with `forbid(unsafe_code)` and zero dependencies.
- [x] Implement import-id parser tests and code.
- [x] Implement nearest `elm.json` lookup tests and code.
- [x] Implement minimal `source-directories` extraction tests and code.
- [x] Implement Elm import dependency scanner tests and code.
- [x] Implement `elm make` command arg builder tests and code.
- [x] Implement temp output compile runner.
- [x] Implement ESM transform tests and code.
- [x] Implement asset transform tests and code.
- [x] Add HMR template injection and nav-key hotfix tests.
- [x] Add thin Vite wrapper with compile serialization.
- [ ] Run parity smoke against current `vite-plugin-elm/example`.
- [ ] Keep GitHub Actions and agent hooks green after every task.

## Loop prompt template

```md
You are working in `rs-vite-plugin-elm`.

Read `PLAN.md` and `AGENT_HARNESS.md`.

Pick the first unchecked task in `AGENT_HARNESS.md`.

Rules:
- stdlib-only Rust
- no unsafe code
- no Rust dependencies
- no npm runtime dependencies except the thin Vite wrapper and official Elm tooling
- one smallest task only
- preserve current `vite-plugin-elm` behavior
- prove with the smallest command
- update `AGENT_HARNESS.md` checkbox/result before stopping

Do not start the next task.
```

## Progress log

- 2026-07-08: Planning docs created from inspection of `vite-plugin-elm` source plus installed `node-elm-compiler`, `elm-esm`, `find-elm-dependencies`, and `find-up` sources. No Rust code implemented.
- 2026-07-08: Added cross-vendor agent entrypoints: `AGENTS.md`, `CLAUDE.md`, and `ANTIGRAVITY.md`. No implementation code added.
- 2026-07-08: Added stdlib-only Rust crate, tests-first core modules, CLI, and npm/Vite shim. Proof: `cargo test`, `npm run build`, Vite hook-level `load` smoke for `Hello.elm`, asset-helper smoke for `Application.elm`, and `npm pack --dry-run` passed. Real `vite-plugin-elm/example npm ci` was attempted but failed because the original plugin package `prepare` script needs root `tsc`; this is unrelated to the Rust package.
- 2026-07-08: Added human README, CI workflow, build badge, and Claude/Codex/Antigravity after-task hooks. Proof: `./scripts/after-task.sh` passed; `act -j build --container-architecture linux/amd64 -P ubuntu-latest=catthehacker/ubuntu:act-latest` passed locally. Macro audit found no repetition where a macro improved readability without hiding simple logic, so no macro was added.
- 2026-07-08: Removed all direct unwrap/expect/panic/todo/unimplemented call sites and added Clippy deny lints for `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, and `dbg_macro`. `./scripts/after-task.sh` now runs `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `npm run build`; CI runs the same gates plus `npm pack --dry-run`.
- 2026-07-08: Speed pass avoided helper-asset scanning when marker text is absent, avoided stdout/stderr allocation on successful Elm compiles, and replaced per-character JSON escape vector allocation with direct `String` pushes. Before profile for `Application.elm` release load: real `0.39,0.13,0.14,0.25,0.13`, user `0.02` each. After profile: real `0.46,0.16,0.12,0.11,0.11`, user `0.01` each. Treat wall time as Elm/cache noisy; guard Rust CPU regression with this workload and after-task gates.
- 2026-07-08: Added harness rule to prefer `const fn` for pure helpers and used it in `options.rs` for compile-mode checks. Added a const-context test for `CompileMode::from_flags`, `is_debug`, and `is_optimize`.
- 2026-07-08: Added a security accusation rubric to the harness and wired `scripts/after-task.sh` to run `scripts/security-post-action.sh` after normal proof gates. The hook is attacker-minded but only promotes findings that satisfy the rubric.

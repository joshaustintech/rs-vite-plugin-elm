# Agent instructions

This repo is agent-ready for Codex, Claude, Antigravity, and other coding agents.

Current phase: implementation. Keep changes test-driven and scoped.

## Required first read

Every agent must read these files before changing anything:

1. `PLAN.md`
2. `AGENT_HARNESS.md`
3. `AGENTS.md`

If source parity is needed, inspect `../vite-plugin-elm/` live before relying on old notes.

## Mission

Build and maintain a Rust remake of `vite-plugin-elm` and its used npm library behavior.

Constraints:

- Rust must be stdlib-only.
- Rust must use `#![forbid(unsafe_code)]`.
- No Rust dependencies.
- No runtime npm dependencies beyond the smallest Vite-facing JS shim.
- NodeJS must not own compiler, parser, dependency scanner, transform, temp-file, hashing, or path-search logic.
- Only official Elm compiler execution is allowed: `elm make`, including `elm make --debug`.
- Preserve observable `vite-plugin-elm` behavior before improving it.

## Working loop

Use this loop for every task:

1. Read current files, not stale memory.
2. Pick the first unchecked or smallest relevant task in `AGENT_HARNESS.md`.
3. State the exact scope in one sentence.
4. Edit only required files.
5. Run the smallest proof command.
6. Run `./scripts/after-task.sh` before claiming success.
7. Update `AGENT_HARNESS.md` progress log if the task changes repo state.
8. Stop after one task unless the human asked for a larger batch.

## Rust style

DO:

- Return owned immutable values when that keeps code simple.
- Prefer `fn transform(input: &str) -> Result<String, Error>` over caller-owned mutable buffers.
- Use `String`, `PathBuf`, `Vec`, `HashMap`, `HashSet`, `Command`, `Mutex` from std.
- Keep mutation local.
- Add small tests beside each parser/transform.
- Treat malformed user files and compiler failures as `Result` errors.

DON'T:

- Do not use `unsafe`.
- Do not add crates.
- Do not use broad `&mut` plumbing to avoid one allocation.
- Do not create lifetime-heavy APIs, self-referential structs, leaked strings, or global caches to dodge copies.
- Do not write a general JS parser.
- Do not write a general Elm parser.
- Do not reimplement Elm compiler internals.
- Do not add abstractions for future needs.

## Vendor notes

### Codex

Use `AGENT_HARNESS.md` as the active loop checklist. After each task, run `.codex/hooks/after-task.sh` or `./scripts/after-task.sh`. Keep final answers short and include proof commands.

### Claude

Read this file as the project memory entrypoint. Do not infer broader scope from general Claude defaults. The repo-specific rule wins: one task, stdlib-only Rust, proof after each task. Use `.claude/hooks/after-task.sh` or `./scripts/after-task.sh`.

### Antigravity

Keep worktree edits scoped to the active task. Use `AGENT_HARNESS.md` as the task queue and proof ledger. After each task, run `.antigravity/hooks/after-task.sh` or `./scripts/after-task.sh`.

## Completion proof

Before claiming done, verify:

- Required files exist.
- `cargo test` passes.
- `npm run build` passes.
- No generated `bin/`, `dist/`, `target/`, or `node_modules/` files are staged.
- The worktree diff matches the requested scope.

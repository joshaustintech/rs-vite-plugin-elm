# rs-vite-plugin-elm plan

Goal: remake `vite-plugin-elm` 3.1.0-2 as a stdlib-only Rust toolchain with no `unsafe` code. Move all current plugin/library logic out of NodeJS. Keep only the smallest Vite-facing JS shim if Vite cannot load a native plugin directly; that shim must contain no compiler, parser, dependency-scanner, or transformer logic. The only external compiler process is official Elm (`elm make`, including `elm make --debug`). Do not port Elm compiler internals.

## Hard constraints

- Rust crate code uses `#![forbid(unsafe_code)]`.
- Rust crate uses stdlib only. No `serde`, no regex crate, no parser crates, no temp-file crates, no path-search crates, no hashing crates.
- Do not shell out to `npm`, `node`, `npx`, `elm-esm`, `node-elm-compiler`, `find-elm-dependencies`, `acorn`, or `find-up`.
- Only external process allowed in the Rust core is the official Elm binary: `elm make ...`.
- Node/Vite wrapper must stay thin and logic-free: normalize Vite hook input, call Rust executable/library boundary, return `{ code, map: null }`, send HMR events.
- Match current observable behavior before improving it.
- No real Rust code until this plan is accepted.

## Source behavior to clone

Current package: `vite-plugin-elm`.

Inspected source inventory:

- `vite-plugin-elm/src/index.ts`: Vite plugin hooks, import-id parsing, `?with` resolution, dependency map, compile lock use, watch-file registration, HMR event dispatch, friendly `-- NO MAIN` rewrite.
- `vite-plugin-elm/src/compiler.ts`: nearest `elm.json` lookup via `find-up`, `node-elm-compiler.compileToString`, `elm-esm.toESModule`, asset injection, custom compiler hook.
- `vite-plugin-elm/src/pluginOptions.ts`: dev/prod defaults for `debug`, `optimize`, `verbose`; custom compiler overrides `nodeElmCompilerOptions`.
- `vite-plugin-elm/src/assetsInjector.ts`: `acorn` AST scan for plain asset tags and `VITE_PLUGIN_HELPER_ASSET` helper calls; SHA-1 import-name generation.
- `vite-plugin-elm/src/hmrInjector.ts`: static Elm hot-reload runtime, Browser.Navigation.Key patch, ports reconnect, scheduler cancellation, dependency accept list.
- `vite-plugin-elm/src/mutex.ts`: single in-process queue to serialize Elm compiler calls because `elm-stuff` cache is not concurrency-safe.
- `node-elm-compiler/dist/index.js`: `elm make` arg builder, temp output file, env/stdout/stderr handling, error messages, `findAllDependencies` re-export.
- `node-elm-compiler/dist/worker.js`: worker helper not used by this plugin; no Rust parity required unless public API is later widened.
- `elm-esm/src/index.js`: regex/string conversion from Elm IIFE output to ESM export.
- `elm-esm/src/run.js`: CLI wrapper not used by this plugin; no Rust parity required.
- `find-elm-dependencies/index.js` and `src/dependencies.js`: first-line module inference, `elm.json` source dir lookup, import-block scanner, recursive module resolution.
- `find-up/index.js`: upward nearest-file search; only sync `findUpSync('elm.json', { cwd: dirname(pathname) })` behavior matters.

Direct runtime deps and replacement target:

- `node-elm-compiler`: builds `elm make` args, sets `LANG=en_US.UTF-8`, writes output to a temp file for `compileToString`, captures stdout/stderr, rejects on nonzero exit, exposes `findAllDependencies` from `find-elm-dependencies`.
- `elm-esm`: converts Elm IIFE JS into ESM by regex/string replacement and appends `export const Elm = ...`.
- `find-elm-dependencies`: reads Elm imports, finds enclosing `elm.json`, resolves `source-directories`, recurses through imports, skips missing modules.
- `find-up`: only used to find nearest `elm.json` from target directory.
- `acorn`/`acorn-walk`: only used to locate asset string literals and helper calls in compiled JS.
- `crypto`: only used for SHA-1-ish stable import names. Rust stdlib has no SHA-1, so use a documented stdlib hash replacement with collision handling in generated names.

Out-of-scope npm library surfaces:

- Do not remake `node-elm-compiler.compileWorker`; `vite-plugin-elm` does not call it.
- Do not remake `elm-esm` CLI; `vite-plugin-elm` only uses `toESModule`.
- Do not remake generic `find-up` matcher/function APIs; only nearest `elm.json` lookup is needed.
- Do not remake generic `acorn` AST behavior; only the exact asset-tag/helper-call detection is needed.

Plugin behavior:

- `parseImportId(id)` accepts `.elm` files unless URL search has `raw`.
- `?with=./Other.elm` compiles multiple main files in one Elm invocation.
- In `load`, resolve `with` paths relative to importer, collect targets, scan dependencies, compile under one global lock, add dependencies as watch files, inject assets, inject HMR in dev, return JS code.
- In `handleHotUpdate`, if changed Elm file is a known dependency of a compilable import, send Vite custom event `hot-update-dependents` and return those module nodes.
- On Elm error containing `-- NO MAIN`, throw friendly “NO MAIN .elm file is requested...” message.

Option behavior:

- `debug`: default `true` unless `NODE_ENV === "production"`.
- `optimize`: default `!debug && NODE_ENV === "production"` unless explicitly boolean.
- `verbose`: `true` in production, false otherwise.
- Pass through existing `nodeElmCompilerOptions` equivalents: `cwd`, `docs`, `debug`, `optimize`, `processOpts`, `report`, `pathToElm`, `verbose`.
- Custom compiler option exists today. For the Rust rewrite, keep the public API only if the thin Node wrapper can call it without adding Node compiler deps; otherwise mark as intentionally unsupported in v1 with a clear migration note.

## Rust module plan

Keep modules boring and small:

- `cli`: parse args from the Node wrapper, read JSON-ish request from stdin only if needed. Prefer simple newline-delimited fields over JSON if the wrapper is internal.
- `plugin_core`: pure orchestration: parse import IDs, resolve targets, find deps, compile, transform JS.
- `elm_make`: build `Command` for `elm make`; create temp output path under `std::env::temp_dir`; capture stdout/stderr; read output; delete temp file best-effort.
- `elm_json`: find closest `elm.json`; extract `source-directories` with a tiny purpose-built parser that only accepts the needed field shape.
- `deps`: scan Elm source import block line by line; infer source root from module name; recurse through source dirs.
- `esm`: convert Elm IIFE to ESM using exact string scanning equivalent to `elm-esm`.
- `assets`: scan compiled ESM for asset tags and helper-call patterns; prepend Vite imports; replace literals/calls with import identifiers.
- `hmr`: keep HMR injection as a static JS template plus dependency interpolation and nav-key hotfix.
- `lock`: one process-local `Mutex<()>` in Node wrapper or Rust server mode. If Rust is one-shot per compile, Node wrapper must serialize calls.
- `errors`: one small error enum with display strings matching current failures.

## Data contracts

Internal compile request fields:

- `id`: original Vite module id.
- `pathname`: absolute Elm path from URL.
- `with`: resolved absolute Elm companion paths.
- `is_build`: bool.
- `debug`: bool.
- `optimize`: bool.
- `verbose`: bool.
- `path_to_elm`: optional path, default `elm`.
- `report`: optional string.
- `docs`: optional string.
- `cwd`: optional path, default nearest `elm.json` parent.

Internal response:

- success: JS code string only.
- failure: stderr/stdout plus normalized message. No stack traces unless wrapper asks for debug logging.

## Exact algorithms

### Import id parsing

Use `std` string parsing:

- Split at `?`.
- Path is before `?`; URL-decode only enough to handle spaces and normal file URLs.
- Valid iff path ends in `.elm` and query has no `raw` key.
- Collect every `with=` query value in order.

### Elm dependency scan

Clone `find-elm-dependencies` behavior:

- Read first line to infer module name.
- If first line is `module X`, `port module X`, or `effect module X`, compute base dir by backing out one segment per module path segment.
- If first line is not a module declaration, use file directory.
- Walk upward to nearest `elm.json` whose `source-directories` contains that base dir.
- Read source dirs as absolute paths.
- Scan imports only after module declaration.
- Stop after import block ends.
- Accept `import Foo.Bar`.
- Resolve to `Foo/Bar.elm` in first source dir containing the file.
- Keep native `Native.*` `.js` compatibility only if current tests prove it still matters; otherwise document as legacy unsupported after parity review.
- Recurse only `.elm`.
- Avoid cycles with `HashSet<PathBuf>`.

### Elm compiler execution

Build args:

```text
make <targets...> --output <temp.js> [--debug] [--optimize] [--report x] [--docs x]
```

Rules:

- Set `LANG=en_US.UTF-8`.
- Use `cwd = nearest elm.json parent` unless explicitly provided.
- Capture stdout/stderr.
- On nonzero exit, return `Compilation failed\n<combined output>`.
- Output suffix follows requested output extension, default `.js`.
- Delete temp output after read; ignore delete failure.

### ESM transform

Clone `elm-esm` string behavior:

- Extract expression from `_Platform_export(<expr>); }(this));`.
- Comment out IIFE opening `(function (scope) {`.
- Comment out `"use strict";`.
- Comment out `_Platform_export`, `_Platform_mergeExports`, and final export call blocks.
- Append `export const Elm = <expr>;`.
- Add focused tests with real compiled Elm JS from fixtures.

### Asset injection

Support both current forms:

- Plain literal: `'[VITE_PLUGIN_ELM_ASSET:/assets/logo.jpg]'`.
- Helper package: compiled call to function identified by literal `VITE_PLUGIN_HELPER_ASSET`.

No JS parser dependency. Use a small scanner:

- Track single/double/template string bounds and escapes.
- Find exact standalone asset-tag string literal.
- For helper support, first find function/declarator pattern emitted by Elm compiler for the helper literal; then find direct calls where the only arg is a string literal.
- If helper call arg is not a plain string literal, return exact error: `Arguments for VitePluginHelper should be just a plain String`.
- Generate import identifiers as `_asset_<stable_hash>`.
- Use `DefaultHasher` plus a collision suffix map. Do not claim SHA-1 parity; only identifier stability within one output matters.

### HMR injection

Keep HMR JS as close as possible to current `hmrInjector.ts`.

Required parity:

- Append HMR only when not build.
- Trim debug warning by replacing `console.warn('Compiled in DEBUG mode` with commented equivalent.
- Determine Elm 0.19.0/0.19.1 symbol prefix.
- Wrap public module `init`.
- Track instances, flags, DOM node, ports, last state, initial state.
- Reconnect ports on swap.
- Preserve Browser.Navigation.Key hotfix.
- Cancel running scheduler bindings on dispose.
- Accept dependency list and `hot-update-dependents` event.

Known current bug to preserve until tested: `removedInstances.forEach((id) => { delete instance[id] })` likely means `instances[id]`. Do not silently “fix” during parity phase.

## Vite wrapper plan

Keep minimal Node ESM file because Vite plugins are JS objects:

- Export default `plugin(userOptions = {})`.
- Maintain `Map<string, Set<string>>` of compilable import id to deps.
- Implement `handleHotUpdate` in Node wrapper because it needs Vite server objects.
- Implement `load(id)` by resolving `with`, then calling Rust binary/core.
- Serialize compile calls in wrapper with a simple promise queue to protect `elm-stuff`.
- Avoid any other npm runtime dependency.
- Keep this wrapper as compatibility glue, not business logic. If Vite gains direct native plugin loading, delete it and call Rust directly.
- Never reintroduce Node libraries for compiling, parsing, dependency scanning, temp files, hashing, path search, or ESM conversion.

If packaging as npm:

- Publish prebuilt Rust binary per platform or compile at install time only if explicit.
- Do not add runtime npm deps.
- Keep TypeScript types tiny and hand-written if needed.

## Test plan

Use fixtures copied from `vite-plugin-elm/example`.

Rust tests:

- option defaults for dev/prod.
- import id parsing: plain, raw, multiple `with`, encoded path.
- nearest `elm.json` lookup.
- `source-directories` parser accepts app `elm.json`.
- dependency scan: direct import, transitive import, missing import skipped, cycle guarded.
- Elm arg generation.
- ESM transform against real Elm output.
- asset injection: plain tag, helper call, duplicate path imports once, dynamic helper arg error.
- HMR nav-key hotfix.
- NO MAIN error normalization.

Integration checks:

- Run `elm make` for each example main.
- Compare exported `Elm` shape with current plugin output.
- Run Vite dev smoke: import `.elm`, HMR dependent update returns right module.
- Run Vite build smoke: no HMR injection, optimize/debug defaults match.

No test framework dependency in Rust plan unless later accepted. Use `cargo test` only.

## Migration steps

1. Freeze parity fixtures: save current plugin outputs for `Hello.elm`, `Application.elm`, raw import, assets, and `?with`.
2. Build Rust crate skeleton with `forbid(unsafe_code)`, stdlib only.
3. Implement pure parsers/scanners first: import id, `elm.json`, Elm imports.
4. Implement `elm make` runner.
5. Implement ESM transform.
6. Implement asset transform.
7. Add HMR template injection.
8. Add tiny Node/Vite wrapper and compile queue.
9. Run fixture comparison against current plugin.
10. Only after parity, document deliberate non-parity such as custom compiler support if dropped.

## DOs

- Do keep code boring: `String`, `PathBuf`, `Vec`, `HashMap`, `HashSet`, `Command`.
- Do return owned values when that removes lifetime fights.
- Do use `&str`/`&Path` at API boundaries, allocate inside only when useful.
- Do prefer immutable data flow: build a new `String` for transforms instead of in-place mutation gymnastics.
- Do keep mutation local and obvious.
- Do use `Mutex`/queue only around Elm compile, not around pure parsing.
- Do match current error strings where users may depend on them.
- Do add one fixture before changing a transform.
- Do delete temp files best-effort.
- Do treat malformed input as a clear error, not panic.

## DON'Ts

- Do not use `unsafe`.
- Do not add crates for JSON, regex, globbing, hashing, temp files, or CLI parsing.
- Do not write a general JS parser.
- Do not write a general Elm parser.
- Do not use `&mut` across broad call chains when returning a fresh immutable value is simpler and usually faster enough.
- Do not thread lifetimes through the design to avoid one small allocation.
- Do not use global mutable state except the compile lock/queue.
- Do not panic on user files, missing files, invalid UTF-8, or Elm compiler failures.
- Do not “improve” HMR logic before parity tests exist.
- Do not implement Rust code in this planning phase.

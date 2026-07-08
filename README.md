# rs-vite-plugin-elm

[![CI](https://github.com/joshaustintech/rs-vite-plugin-elm/actions/workflows/ci.yml/badge.svg)](https://github.com/joshaustintech/rs-vite-plugin-elm/actions/workflows/ci.yml)

Stdlib-only, safe Rust remake of `vite-plugin-elm`.

Rust owns Elm compilation orchestration, dependency scanning, ESM conversion, asset injection, and HMR injection. NodeJS is only a small Vite plugin shim.

## What this is

`rs-vite-plugin-elm` is a Vite plugin for Elm projects. It keeps Vite-facing JavaScript tiny and moves plugin behavior into a safe Rust binary:

- `.elm` import handling
- `elm make` orchestration
- Elm dependency scanning for Vite watch/HMR
- Elm IIFE-to-ESM conversion
- Vite asset-tag injection
- development HMR injection

The Rust crate is intentionally boring:

- no Rust dependencies
- no `unsafe`
- no custom Elm compiler
- no general JavaScript parser
- no runtime npm dependencies beyond Vite as a peer

## Requirements

- Rust stable toolchain
- Node.js 20 or newer
- npm
- Elm 0.19.1 on `PATH`

## Build

```sh
npm run build
```

## Test

```sh
cargo test
```

Run the same post-task proof agents are expected to run:

```sh
./scripts/after-task.sh
```

## Use from Vite

Install locally while developing:

```sh
npm install /path/to/rs-vite-plugin-elm
```

Then use it in `vite.config.js`:

```js
import elmPlugin from 'rs-vite-plugin-elm'

export default {
  plugins: [elmPlugin()],
}
```

Import Elm as usual:

```js
import { Elm } from './Main.elm'

Elm.Main.init({
  node: document.getElementById('app'),
})
```

## Package Dry Run

Build and inspect the npm package:

```sh
npm run build
npm pack --dry-run
```

Generated package output lives in `bin/` and `dist/`; both are ignored by git.

## Agent Workflow

Agents must keep every task boring and proved:

1. Read `AGENTS.md`.
2. Make one scoped change.
3. Run `./scripts/after-task.sh`.
4. Do not commit generated `bin/`, `dist/`, `target/`, or `node_modules/`.

Read:

- [AGENTS.md](AGENTS.md) for cross-vendor agent instructions.
- [PLAN.md](PLAN.md) for the 1:1 rewrite plan.
- [AGENT_HARNESS.md](AGENT_HARNESS.md) for the looped agent workflow.

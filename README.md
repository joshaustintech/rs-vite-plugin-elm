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

Clone the repo somewhere on your machine:

```sh
git clone https://github.com/joshaustintech/rs-vite-plugin-elm.git
```

Then point your app at that local checkout. If the plugin repo sits next to your app, the path can be relative:

```sh
npm install ../rs-vite-plugin-elm
```

If the repo lives elsewhere, use its absolute path instead:

```sh
npm install /absolute/path/to/rs-vite-plugin-elm
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

That keeps your app running against the local checkout, so edits in the plugin repo are used as soon as you rebuild your app.

## Configuration

`elmPlugin(options)` has four top-level options. `debug` and `optimize` select
Elm compiler modes; `compiler` replaces compilation; `nodeElmCompilerOptions`
is the compatibility namespace for compiler settings.

```ts
import elmPlugin from 'rs-vite-plugin-elm'

export default {
  plugins: [elmPlugin({ debug: false, optimize: true })],
}
```

### `debug?: boolean`

Controls Elm's `--debug` mode. It defaults to `true` outside a production Vite
build and `false` for `NODE_ENV=production`. Debug wins over optimize if both
are true.

Use it to keep Elm debugger support while diagnosing a production-only issue:

```ts
elmPlugin({ debug: true })
```

### `optimize?: boolean`

Controls Elm's `--optimize` mode. Its default is `true` only for production
when debug is off. Optimized builds reject Elm `Debug` usage.

Use an unoptimized production-shaped build when instrumentation must remain:

```ts
elmPlugin({ debug: false, optimize: false })
```

### `compiler?: { compile(targets: string[]): Promise<string> }`

Replaces `elm make`. The plugin still scans Elm dependencies and applies its
ESM, asset, and development-HMR postprocessing. The function receives absolute
Elm entry paths and must return JavaScript suitable for that postprocessing.

Use it when compilation is owned by a remote cache or a controlled build
service:

```ts
elmPlugin({
  compiler: {
    async compile(targets) {
      return fetch('/internal/elm-compile', {
        method: 'POST',
        body: JSON.stringify({ targets }),
      }).then((response) => response.text())
    },
  },
})
```

`compiler` takes precedence over `nodeElmCompilerOptions`.

### `nodeElmCompilerOptions`

This object preserves the familiar `node-elm-compiler` option names for the
built-in `elm make` path:

```ts
elmPlugin({
  nodeElmCompilerOptions: {
    cwd: './frontend',
    pathToElm: '/opt/elm/0.19.1/bin/elm',
    report: 'json',
  },
})
```

- `cwd?: string` — compiler working directory and Elm project root. Use it
  when Vite config lives above the directory containing `elm.json`.
- `docs?: string` — destination passed as `elm make --docs <path>`. Use it to
  emit package documentation during a build.
- `debug?: boolean` — overrides top-level debug for the built-in compiler.
  Use it when compiler policy belongs in one shared option object.
- `optimize?: boolean` — overrides top-level optimize for the built-in
  compiler. Debug still wins if both resolve to true.
- `pathToElm?: string` — Elm executable name or path; defaults to `elm`. Use
  it to pin a managed Elm toolchain.
- `report?: string` — value passed as `elm make --report <format>`. Use
  `json` when another tool consumes compiler diagnostics.
- `verbose?: boolean` — retained for `node-elm-compiler` API compatibility.
  The Rust helper captures compiler output, so it does not currently add
  console logging.
- `processOpts?: Record<string, string>` — retained for API compatibility.
  Upstream `compileToString` replaces these with piped stdio, and this helper
  likewise does not apply arbitrary child-process options. Do not rely on it
  for environment or stdio control.

The built-in compiler defaults are `pathToElm: 'elm'`, `verbose: false` in
development, and `verbose: true` in production. Omitted `cwd` resolves from
the nearest `elm.json` for the imported entry file.

### Multi-main imports

This is import syntax rather than an `elmPlugin` option. Repeated `with`
parameters compile several Elm entry modules together, avoiding duplicated Elm
runtime and shared-module output:

```ts
import { Elm } from './App.elm?with=./Admin.elm&with=./Embed.elm'

Elm.App.init({ node: document.querySelector('#app') })
Elm.Admin.init({ node: document.querySelector('#admin') })
```

### Assets and development HMR

These are built-in import transformations, not configuration options. A
standalone Elm string tagged as an asset becomes a Vite asset import:

```elm
Html.img [ Html.Attributes.src "[VITE_PLUGIN_ELM_ASSET:/assets/logo.svg]" ] []
```

In development the plugin tracks imported Elm dependencies, registers them
with Vite, and sends dependent modules through its HMR path. Production output
omits that runtime injection.

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

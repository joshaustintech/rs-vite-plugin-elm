# rs-vite-plugin-elm

Stdlib-only, safe Rust remake of `vite-plugin-elm`.

Rust owns Elm compilation orchestration, dependency scanning, ESM conversion, asset injection, and HMR injection. NodeJS is only a small Vite plugin shim.

## Build

```sh
npm run build
```

## Test

```sh
cargo test
```

## Use from Vite

```js
import elmPlugin from 'rs-vite-plugin-elm'

export default {
  plugins: [elmPlugin()],
}
```

Read:

- [AGENTS.md](AGENTS.md) for cross-vendor agent instructions.
- [PLAN.md](PLAN.md) for the 1:1 rewrite plan.
- [AGENT_HARNESS.md](AGENT_HARNESS.md) for the looped agent workflow.

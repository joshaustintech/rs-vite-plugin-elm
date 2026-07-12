# Roadmap

## Line-count reductions retained

Each retained change passed the existing parity and build gates. The pass
removed 29 lines from the edited tracked files without changing behavior.

1. Shared CLI path-array JSON serialization across `load` and `deps`.
2. Ordered compiler-option key list for JS-to-Rust argument forwarding.
3. Inlined one-use POSIX path conversion helper.
4. Replaced duplicated harness prompt prose with a link to `AGENTS.md`.

The other six candidates were attempted and rejected: optional parsers had no
safe net reduction; collapsing `CompileRequest` risks its public API; shared
dependency normalization adds a helper for only two call sites; packaging
already copies one JS source into `dist`; environment-check consolidation
would blur distinct error contracts; and HMR is already one static template.

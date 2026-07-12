# Roadmap

## Line-count reduction candidates

These are candidates, not approved behavior changes. Each needs parity proof
before adoption.

1. Replace duplicated CLI optional-value parsers with one typed decoder.
2. Combine repeated JSON escaping/output assembly in CLI commands.
3. Collapse `CompileRequest` forwarding into a single compile entry helper.
4. Share dependency normalization between custom and built-in wrapper paths.
5. Generate thin JS/dist copies from one source during packaging.
6. Merge wrapper environment checks into one command-resolution function.
7. Replace hand-written repeated option forwarding with one ordered field list.
8. Consolidate HMR static template fragments into one literal plus substitutions.
9. Fold small one-use path helpers into callers where tests keep intent clear.
10. Remove duplicated plan/harness workflow prose by making one canonical rule set and linking to it.

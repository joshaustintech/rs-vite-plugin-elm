# Claude entrypoint

Read `AGENTS.md`, then `PLAN.md`, then `AGENT_HARNESS.md`.

Follow the repo loop: one scoped task, stdlib-only Rust, no unsafe code, no dependencies, tests first, smallest proof command.

After each task, run:

```sh
.claude/hooks/after-task.sh
```

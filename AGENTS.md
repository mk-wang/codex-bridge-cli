---
owner: codex-bridge maintainers
status: active
updated: 2026-06-27
tags:
  - agent-instructions
links:
  - README.md
  - docs/codex-bridge-cli-design.md
  - docs/progress-plan.md
---

# Agent Instructions

This is a greenfield Rust CLI project for `codex-bridge`, a local protocol
bridge from Codex Responses API to upstream Chat Completions.

## Required Reading

Before making changes, read:

1. `README.md`
2. `docs/codex-bridge-cli-design.md`
3. `docs/progress-plan.md`

`docs/progress-plan.md` is the durable plan mode and progress tracker. Update it
whenever implementation status, scope, blockers, or verification results change.

## Project Shape

- CLI entrypoint: `src/main.rs`
- HTTP shell: `src/bridge.rs`
- Config: `src/config.rs`
- Translator modules: `src/proxy/`
- Example config: `examples/codex-bridge.yaml`

Keep HTTP concerns in `src/bridge.rs`. Keep protocol translation behavior in
`src/proxy/`.

## Verification

Run the narrowest useful command while iterating, then finish with:

```bash
cargo fmt
cargo check
cargo test
```

Run `cargo build --release` when startup or packaging behavior changes.

## Documentation Rules

- Treat this as a greenfield project.
- Do not describe implementation work as a migration path option.
- Keep `README.md` short and user-facing.
- Keep design facts in `docs/codex-bridge-cli-design.md`.
- Keep status and next work in `docs/progress-plan.md`.

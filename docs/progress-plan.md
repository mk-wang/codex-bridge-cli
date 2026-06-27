---
owner: codex-bridge maintainers
status: active
updated: 2026-06-27
tags:
  - planning
  - delivery
links:
  - codex-bridge-cli-design.md
---

# Progress Plan

This is the single source of truth for implementation progress. Update it when
work changes project status, scope, or verification results.

## Plan Mode Protocol

Use this file as the durable plan mode:

1. Before starting work, read this file and identify the first unchecked item
   whose dependencies are satisfied.
2. During work, update the item status if it changes materially.
3. After implementation, add the exact verification commands and result.
4. Do not mark an item done unless it is implemented and verified.
5. If a new blocker appears, add it under `Open Questions / Blockers`.

Status vocabulary:

- `todo`: not started.
- `doing`: active in the current work session.
- `done`: implemented and verified.
- `blocked`: cannot proceed without external input or environment.

## Current Snapshot

Updated: 2026-06-27

| Area | Status | Evidence |
|---|---|---|
| Rust CLI scaffold | done | `cargo check` passes |
| Axum HTTP server | done | `/health` checked locally |
| YAML config parsing | done | `examples/codex-bridge.yaml` loads |
| Responses to Chat request conversion | done | translator unit tests pass |
| Chat JSON to Responses JSON conversion | done | translator unit tests pass |
| Chat SSE to Responses SSE conversion | done | translator unit tests pass |
| In-memory history backfill | done | translator unit tests pass |
| Release binary build | done | `cargo build --release` passes |
| Real LiteLLM integration | todo | not run |
| Real Codex CLI integration | todo | not run |
| Production startup/logging | todo | not implemented |

## Milestones

### M0: Greenfield CLI Foundation

Status: done

- [x] Create Cargo project.
- [x] Add `codex-bridge` binary.
- [x] Add YAML config.
- [x] Add example config.
- [x] Add `/health`.

Verification:

```bash
cargo check
target/release/codex-bridge --config examples/codex-bridge.yaml
curl -sS http://127.0.0.1:4010/health
```

Observed `/health` response:

```json
{"status":"ok","service":"codex-bridge"}
```

### M1: Translation Core

Status: done

- [x] Responses request to Chat request conversion.
- [x] Chat non-streaming response to Responses response conversion.
- [x] Chat SSE to Responses SSE conversion.
- [x] Inline think handling.
- [x] Reasoning extraction and restoration.
- [x] Namespace/custom/tool-search restoration.
- [x] Canonical JSON handling.
- [x] History enrichment and stream recording.

Verification:

```bash
cargo test
```

Last result: 96 passed, 0 failed.

### M2: Real Upstream Integration

Status: todo

- [ ] Start LiteLLM on `127.0.0.1:4000`.
- [ ] Start bridge on `127.0.0.1:4010`.
- [ ] Verify `GET /v1/models` through bridge.
- [ ] Send one non-streaming `/v1/responses` request through bridge.
- [ ] Send one streaming `/v1/responses` request through bridge.
- [ ] Capture representative request/response fixtures for regression tests.

Acceptance:

- Bridge forwards auth correctly.
- Upstream Chat request is accepted.
- Codex-compatible Responses JSON/SSE is returned.
- No malformed SSE or JSON conversion errors in logs.

### M3: Codex CLI Integration

Status: todo

- [ ] Configure Codex model provider with bridge base URL.
- [ ] Confirm `wire_api = "responses"`.
- [ ] Run a simple Codex prompt with no tools.
- [ ] Run a prompt that triggers at least one tool call.
- [ ] Run a multi-turn tool-call follow-up to validate history backfill.

Acceptance:

- Codex completes a normal answer.
- Codex completes a tool-using turn.
- Follow-up request does not fail because of missing prior tool-call context.

### M4: Config Completion

Status: done

- [x] Wire `history.max_cached_responses` into `CodexChatHistoryStore`.
- [x] Add optional YAML reasoning capability config.
- [x] Decide whether `/responses/compact` needs distinct handling.
- [x] Add config validation errors for empty upstream URL and invalid endpoint.

Decision — `/responses/compact`: Forwarded to the same handler as
`/v1/responses`. The compact endpoint compacts prior turns server-side before
the request arrives; the bridge receives only the compacted form and applies
its standard history enrichment. No separate handling is required.

Verification:

```bash
cargo test
```

New tests: `config::tests::*` (6 tests covering validation and new fields),
`codex_chat_history::tests::with_capacity_evicts_oldest_when_full`.
Total: 103 passed, 0 failed.


### M5: Operational Readiness

Status: done

- [x] Add structured logging.
- [x] Add startup script for local use.
- [x] Add documented install/build path.
- [x] Add graceful shutdown behavior.
- [x] Add troubleshooting section for upstream 400/401/502/SSE failures.

Implementation notes:

- Replaced `log` crate with `tracing` + `tracing-subscriber` (env-filter).
  Startup emits `info` events; per-request debug; upstream errors emit `warn`.
- `serve_with_graceful_shutdown` listens for SIGINT and SIGTERM; in-flight
  requests drain before the process exits.
- `scripts/run.sh` builds the release binary if missing, then launches.
- `README.md` has Requirements, Install, Run, Logging, and Troubleshooting.

Verification:

```bash
cargo fmt && cargo check && cargo test
cargo build --release
```

Results: 103 passed, 0 failed. Release binary: `target/release/codex-bridge`.


## Open Questions / Blockers

- What exact LiteLLM config will be used for the first end-to-end test?
- Should `/responses/compact` be a supported endpoint or explicitly return
  `501 Not Implemented` until semantics are defined?
- Which models need explicit reasoning capability YAML rather than default
  translator behavior?

## Last Verification

2026-06-27:

```bash
cargo fmt
cargo check
cargo test
cargo build --release
```

Results:

- `cargo check`: passed.
- `cargo test`: 96 passed, 0 failed.
- `cargo build --release`: passed.
- Release binary: `target/release/codex-bridge`.

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
| Real LiteLLM integration | done | M2 passed through `glm-5-2` |
| Real Codex CLI integration | done | M3 passed with Codex CLI using `glm-5-2` |
| Production startup/logging | done | M5 verified |

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

Status: done

- [x] Start LiteLLM on `127.0.0.1:4000`.
- [x] Start bridge on `127.0.0.1:4010`.
- [x] Verify `GET /v1/models` through bridge.
- [x] Send one non-streaming `/v1/responses` request through bridge.
- [x] Send one streaming `/v1/responses` request through bridge.
- [x] Capture representative request/response fixtures for regression tests.

Environment:

- LiteLLM config: `/Users/mk/.litellm/opencode-bridge.yaml`
- LiteLLM listener: `127.0.0.1:4000`
- Bridge config: `examples/codex-bridge.yaml`
- Bridge listener: `127.0.0.1:4010`
- Test model alias: `glm-5-2`
- Auth source: `LITELLM_MASTER_KEY`

Verification:

```bash
curl -sS -m 10 -H "Authorization: Bearer $LITELLM_MASTER_KEY" \
  http://127.0.0.1:4000/v1/models

RUST_LOG=debug target/release/codex-bridge \
  --config examples/codex-bridge.yaml

curl -sS -m 10 -w '\nHTTP_STATUS:%{http_code}\n' \
  http://127.0.0.1:4010/v1/models

curl -sS -m 60 -w '\nHTTP_STATUS:%{http_code}\n' \
  -H 'Content-Type: application/json' \
  -X POST http://127.0.0.1:4010/v1/responses \
  -d '{"model":"glm-5-2","input":"Reply with exactly: bridge nonstream ok","stream":false,"max_output_tokens":64}'

curl -sS -N -m 60 -w '\nHTTP_STATUS:%{http_code}\n' \
  -H 'Content-Type: application/json' \
  -X POST http://127.0.0.1:4010/v1/responses \
  -d '{"model":"glm-5-2","input":"Reply with exactly: bridge stream ok","stream":true,"max_output_tokens":64}'
```

Results:

- Direct LiteLLM `/v1/models` with bearer auth returned model aliases:
  `ds-v4-pro`, `ds-v4-flash`, `glm-5-2`, `gemini-3.5-flash`.
- Bridge `/v1/models` returned HTTP 200 and the same model list.
- Bridge non-streaming `/v1/responses` returned HTTP 200 with
  `status: "completed"` and output text `bridge nonstream ok`.
- Bridge streaming `/v1/responses` returned HTTP 200 with valid Responses SSE
  events ending in `response.completed` and output text `bridge stream ok`.
- Fixtures captured under `tests/fixtures/m2-real-upstream/`.

Acceptance:

- Bridge forwards auth correctly.
- Upstream Chat request is accepted.
- Codex-compatible Responses JSON/SSE is returned.
- No malformed SSE or JSON conversion errors in logs.

### M3: Codex CLI Integration

Status: done

- [x] Configure Codex model provider with bridge base URL.
- [x] Confirm `wire_api = "responses"`.
- [x] Run a simple Codex prompt with no tools.
- [x] Run a prompt that triggers at least one tool call.
- [x] Run a multi-turn tool-call follow-up to validate history backfill.

Environment:

- Bridge config: `examples/codex-bridge.yaml`
- Bridge listener: `127.0.0.1:4010`
- Codex CLI: `/opt/homebrew/bin/codex`
- Codex CLI version: `0.142.3`
- Test model alias: `glm-5-2`
- Provider config: command-line overrides only; no persistent
  `~/.codex/config.toml` changes.
- Provider base URL: `http://127.0.0.1:4010`
- Provider wire API: `responses`

Provider verification:

```bash
codex doctor --summary --ascii \
  -c 'model="glm-5-2"' \
  -c 'model_provider="codex-bridge"' \
  -c 'model_providers.codex-bridge={name="Codex Bridge", base_url="http://127.0.0.1:4010", wire_api="responses"}'
```

Result: 17 ok, 1 idle, 0 warn, 0 fail. Doctor reported that OpenAI auth is not
required for the active model provider and that active provider endpoints are
reachable over HTTP.

Simple prompt:

```bash
codex exec --json --skip-git-repo-check \
  -c 'model="glm-5-2"' \
  -c 'model_provider="codex-bridge"' \
  -c 'model_providers.codex-bridge={name="Codex Bridge", base_url="http://127.0.0.1:4010", wire_api="responses"}' \
  -c 'sandbox_mode="read-only"' \
  -c 'approval_policy="never"' \
  'Reply with exactly: codex bridge simple ok'
```

Result: turn completed; final agent message was `codex bridge simple ok`.

Tool prompt:

```bash
codex exec --json --skip-git-repo-check \
  -c 'model="glm-5-2"' \
  -c 'model_provider="codex-bridge"' \
  -c 'model_providers.codex-bridge={name="Codex Bridge", base_url="http://127.0.0.1:4010", wire_api="responses"}' \
  -c 'sandbox_mode="read-only"' \
  -c 'approval_policy="never"' \
  'You must use the shell tool exactly once to run: printf codex-bridge-tool-ok. Then reply with exactly the command output and nothing else.'
```

Result: Codex emitted a `command_execution`, ran
`/opt/homebrew/bin/zsh -lc 'printf codex-bridge-tool-ok'`, received exit code
0, and returned `codex-bridge-tool-ok`.

Follow-up turn:

```bash
codex exec resume --last --json --skip-git-repo-check \
  -c 'model="glm-5-2"' \
  -c 'model_provider="codex-bridge"' \
  -c 'model_providers.codex-bridge={name="Codex Bridge", base_url="http://127.0.0.1:4010", wire_api="responses"}' \
  -c 'sandbox_mode="read-only"' \
  -c 'approval_policy="never"' \
  'Based on the previous command result, reply with exactly: codex-bridge-followup-ok'
```

Result: resumed the same thread as the tool turn and completed with
`codex-bridge-followup-ok`; no tool-history conversion error occurred.

Fixtures:

- `tests/fixtures/m3-codex-cli/simple-turn.jsonl`
- `tests/fixtures/m3-codex-cli/tool-turn.jsonl`
- `tests/fixtures/m3-codex-cli/followup-turn.jsonl`

Residual note:

- Codex CLI logs a non-blocking metadata warning because its model refresh path
  expects a top-level `models` catalog response, while the bridge currently
  proxies LiteLLM's OpenAI-style `/v1/models` response with top-level `data`.
  Turns still complete using fallback model metadata.

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

- No current milestone blockers.
- Follow-up: decide whether the bridge should synthesize a Codex-compatible
  model catalog response to remove the Codex CLI fallback metadata warning.

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
- `cargo test`: 103 passed, 0 failed.
- `cargo build --release`: passed.
- Release binary: `target/release/codex-bridge`.

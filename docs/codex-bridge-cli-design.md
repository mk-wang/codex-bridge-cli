---
owner: codex-bridge maintainers
status: active
updated: 2026-06-27
tags:
  - architecture
  - protocol-bridge
links:
  - progress-plan.md
---

# codex-bridge-cli Design

## Purpose

`codex-bridge` is a local protocol bridge for Codex CLI. It accepts OpenAI
Responses API requests from Codex, converts them to OpenAI Chat Completions
requests, forwards them to a single upstream Chat Completions compatible
service, then maps the result back to Responses format.

```text
Codex CLI --/v1/responses--> codex-bridge --/v1/chat/completions--> Chat-compatible upstream
```

The bridge owns protocol translation only. Routing, model fallback, account
selection, billing, and upstream provider policy remain outside this binary.
LiteLLM is one supported upstream shape, not a requirement; any service that
accepts OpenAI-style Chat Completions JSON and SSE can be used.

## Status Source

Implementation status, verification results, next milestones, and blockers live
in [progress-plan.md](progress-plan.md). Keep this document focused on stable
architecture and protocol boundaries.

## Module Boundaries

### HTTP Shell

`src/bridge.rs` is the only place that should know about axum, reqwest, route
registration, upstream URLs, headers, and HTTP response bodies.

Responsibilities:

- Read inbound Responses JSON.
- Enrich request history before conversion.
- Build tool context from the original Responses request.
- Convert request JSON to Chat Completions JSON.
- Forward to the configured upstream Chat endpoint.
- Convert non-streaming Chat JSON responses back to Responses JSON.
- Convert streaming Chat SSE responses back to Responses SSE.
- Record tool-call output items into in-memory history.

The HTTP shell should stay thin. Translation rules belong in the translation
modules, not in route handlers.

### Translation Core

The translation core lives under `src/proxy/`:

- `providers/transform_codex_chat.rs`
  - Responses request to Chat request.
  - Chat non-streaming response to Responses response.
  - Error shape normalization.
  - Tool context creation and tool-name restoration.
- `providers/streaming_codex_chat.rs`
  - Chat SSE to Responses SSE state machine.
  - Text, reasoning, inline `<think>`, usage, and tool-call event sequencing.
- `providers/codex_chat_history.rs`
  - In-memory response history.
  - Follow-up request enrichment for tool outputs missing their prior calls.
- `providers/codex_chat_common.rs`
  - Shared reasoning extraction and response item helpers.
- `sse.rs`
  - SSE block parsing and UTF-8-safe buffering.
- `json_canonical.rs`
  - Canonical JSON and tool-argument normalization.
- `error.rs`
  - Bridge error type and JSON error responses.

`src/provider.rs` intentionally contains only the reasoning capability
descriptor needed by the translator. It is not a provider router.

### Configuration

Example:

```yaml
upstream:
  base_url: http://127.0.0.1:4000
  api_key_env: UPSTREAM_API_KEY
  timeout: 300
  chat_endpoint: /v1/chat/completions
server:
  host: 127.0.0.1
  port: 4010
history:
  max_cached_responses: 512
```

Runtime behavior:

- `upstream.base_url` and `upstream.chat_endpoint` form the Chat Completions
  target URL. Both are validated at startup: `base_url` must be non-empty;
  `chat_endpoint` must be non-empty and start with `/`.
- `upstream.api_key_env` is read at request time. If present and non-empty, it
  is sent as bearer auth.
- If no configured API key is available, inbound `Authorization` is forwarded.
- `server.host` and `server.port` control the listener.
- `history.max_cached_responses` caps the in-memory response cache. The
  configured value is wired through to `CodexChatHistoryStore::with_capacity`;
  oldest responses are evicted when the limit is reached.
- `reasoning` is an optional section that overrides the translator's built-in
  model-name-based reasoning capability inference. Useful when models are
  served under custom aliases by an upstream gateway. When the section is absent
  (or all fields are null), the translator falls back to its default inference.

## Request Flow

1. Codex sends a Responses request to `/v1/responses`.
2. `CodexChatHistoryStore::enrich_request` restores prior tool-call context
   when possible.
3. `build_codex_tool_context_from_request` records mappings needed to restore
   namespace, custom, and tool-search calls.
4. `responses_to_chat_completions_with_reasoning` converts the request to Chat
   Completions.
5. The bridge forwards JSON to the configured upstream.
6. For non-streaming responses, `chat_completion_to_response_with_context`
   converts the response back and the store records tool calls.
7. For streaming responses,
   `create_responses_sse_stream_from_chat_with_context` converts SSE chunks and
   `record_responses_sse_stream` records completed tool-call items.

## Endpoints

| Path | Method | Behavior |
|---|---|---|
| `/health` | GET | Returns `{"status":"ok","service":"codex-bridge"}`. |
| `/v1/responses` | POST | Main Responses → Chat Completions bridge route. |
| `/responses` | POST | Alias for `/v1/responses`. |
| `/v1/responses/compact` | POST | Forwarded to the same handler as `/v1/responses`. The Responses `/compact` endpoint compacts prior conversation turns into a single `previous_response_id`. The bridge receives only the final compacted form; no special handling is needed beyond the existing history enrichment. |
| `/responses/compact` | POST | Alias for `/v1/responses/compact`. |
| `/v1/models` | GET | Fetches `upstream.base_url/v1/models` and returns a hybrid model catalog: the upstream OpenAI-style `data` field is preserved, and a Codex-compatible top-level `models` field is synthesized when possible. |
| `/models` | GET | Alias for `/v1/models`. |

## Invariants

- The bridge speaks Responses to Codex and Chat Completions to the upstream.
- The bridge has exactly one upstream target per process.
- Protocol translation must be deterministic for the same request/response.
- Tool call names flattened for Chat must be restorable to Responses output
  items.
- Streaming output must emit valid Responses SSE event order.
- UTF-8 split boundaries in upstream SSE chunks must not corrupt data.
- History is in-memory only; process restart clears it.
- Config is validated at startup; invalid fields cause a hard exit with an
  actionable error message.

## Out Of Scope

- Anthropic Messages protocol translation.
- Gemini native protocol translation.
- MCP proxying.
- Codex OAuth or account management.
- Upstream gateway source patches.
- Persistent usage/billing storage.
- Built-in provider failover.
- Disk persistence for `previous_response_id` history.

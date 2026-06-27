# codex-bridge-cli

Lightweight local bridge for Codex CLI:

```text
Codex CLI --/v1/responses--> codex-bridge --/v1/chat/completions--> Chat-compatible upstream
```

The project is a greenfield Rust CLI with a small HTTP shell and isolated
translation modules. The upstream can be any service that accepts
OpenAI-compatible Chat Completions requests, such as LiteLLM, OpenAI-compatible
model gateways, or self-hosted Chat endpoints.

Project docs:

- [Design](docs/codex-bridge-cli-design.md)
- [Progress plan](docs/progress-plan.md)
- [Workflow](docs/workflow.md)

## GUI Manager

If you are looking for a graphical interface to manage Codex CLI and other AI coding agents (such as Claude Code, Gemini CLI, OpenCode, OpenClaw, and Hermes Agent), check out **[cc-switch](https://github.com/farion1231/cc-switch)**: an open-source, cross-platform desktop All-in-One assistant.

## Requirements

- Rust 1.85.0 or newer. Install via [rustup](https://rustup.rs/).

## Install

```bash
git clone <repo-url> codex-bridge-cli
cd codex-bridge-cli
cargo build --release
```

The release binary is placed at `target/release/codex-bridge`.

## Run

```bash
# Build and start with the default example config:
./scripts/run.sh

# Or point at a custom config:
./scripts/run.sh /path/to/my-config.yaml

# Or run the binary directly:
target/release/codex-bridge --config examples/codex-bridge.yaml
```

The default example listens on `127.0.0.1:4010` and forwards Chat Completions to
`http://127.0.0.1:4000/v1/chat/completions`. Point `upstream.base_url` and
`upstream.chat_endpoint` at any Chat Completions compatible service.

The example config reads upstream bearer auth from `UPSTREAM_API_KEY`. To
forward the inbound `Authorization` header from Codex instead, omit
`upstream.api_key_env` in your config.

Stop the bridge with `Ctrl-C` (SIGINT) or `kill -TERM <pid>`. The process
drains in-flight requests before exiting.

## Logging

The bridge uses structured logging via `tracing`. Set the `RUST_LOG` environment
variable to control verbosity:

```bash
RUST_LOG=debug target/release/codex-bridge --config examples/codex-bridge.yaml
```

Default level is `info`. Valid levels: `error`, `warn`, `info`, `debug`, `trace`.

## Codex Config

Point Codex at the bridge and keep the Responses wire API:

```toml
[model_providers.codex-bridge]
base_url = "http://127.0.0.1:4010"
wire_api = "responses"
```

## Verify

```bash
cargo check
cargo test
```

The translator tests cover request conversion, non-streaming response
conversion, Chat SSE to Responses SSE conversion, history backfill,
config validation, and canonical JSON handling.

## Troubleshooting

### Upstream 400 Bad Request

The Chat Completions request was rejected. Common causes:

- The model name is not recognized by the upstream provider.
- The request contains fields not accepted by the upstream (e.g. unsupported
  reasoning params). Set `RUST_LOG=debug` to log the outgoing Chat request.
- The upstream gateway requires a provider-qualified model name, e.g.
  `openai/gpt-4o` rather than `gpt-4o`.

### Upstream 401 Unauthorized

The API key is missing or wrong. Verify:

- `upstream.api_key_env` in your config matches an environment variable that
  is set and non-empty.
- If you omit `api_key_env`, the inbound `Authorization` header from Codex is
  forwarded. Make sure Codex is configured with a valid key.

### Upstream 502 Bad Gateway / connection refused

The Chat Completions upstream is not reachable at `upstream.base_url`. Check:

- The upstream service is running and listening on the configured address.
- `upstream.base_url` does not have a trailing slash (e.g.
  `http://127.0.0.1:4000`, not `http://127.0.0.1:4000/`).
- Firewall or port binding issues if running in a container.

### Malformed SSE / JSON errors in streaming

- Set `RUST_LOG=debug` and inspect the raw SSE chunks in the log.
- If the upstream sends `data: [DONE]` without a preceding `response.completed`
  event, the bridge closes the stream cleanly — this is expected.
- Partial UTF-8 sequences across SSE chunk boundaries are handled
  transparently; if you see corruption, file an issue with the raw bytes.

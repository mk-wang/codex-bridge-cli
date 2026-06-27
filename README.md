# codex-bridge-cli

Lightweight local bridge for Codex CLI:

```text
Codex CLI --/v1/responses--> codex-bridge --/v1/chat/completions--> LiteLLM
```

The project is a greenfield Rust CLI with a small HTTP shell and isolated
translation modules.

Project docs:

- [Design](docs/codex-bridge-cli-design.md)
- [Progress plan](docs/progress-plan.md)
- [Workflow](docs/workflow.md)

## Build

```bash
cargo build --release
```

## Run

```bash
target/release/codex-bridge --config examples/codex-bridge.yaml
```

The default example listens on `127.0.0.1:4010` and forwards Chat Completions to
LiteLLM at `http://127.0.0.1:4000/v1/chat/completions`.

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

The translator tests currently cover request conversion, non-streaming response
conversion, Chat SSE to Responses SSE conversion, history backfill, and
canonical JSON handling.

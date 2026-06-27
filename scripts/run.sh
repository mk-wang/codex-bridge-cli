#!/usr/bin/env bash
# Run codex-bridge with the example config.
# Usage: ./scripts/run.sh [path/to/config.yaml]
# Requires: cargo build --release (or the binary already in target/release/).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
CONFIG="${1:-"$REPO_ROOT/examples/codex-bridge.yaml"}"
BINARY="$REPO_ROOT/target/release/codex-bridge"

if [[ ! -f "$BINARY" ]]; then
  echo "Binary not found. Building release binary..." >&2
  cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml"
fi

exec "$BINARY" --config "$CONFIG"

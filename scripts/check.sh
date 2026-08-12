#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
uv run external-tests/run.py --serial
(
  cd editors/vscode
  npm test
)

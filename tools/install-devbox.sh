#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "Installing yamark CLI with cargo install --path ."
(
  cd "$repo_root"
  cargo install --path .
)

echo "Installing Yamark editor extension"
YAMARK_BUNDLE="${YAMARK_BUNDLE:-1}" "$repo_root/editors/vscode/scripts/install-local.sh"

#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../../.." && pwd)"
code_bin="${CODE_BIN:-code}"
out="${VSIX_OUT:-$repo_root/target/vscode/yamark-dev.vsix}"

VSIX_OUT="$out" "$script_dir/package-dev.sh" >/dev/null
NODE_NO_WARNINGS=1 "$code_bin" --install-extension "$out" --force

cat <<'EOF'
Installed Yamark extension.

Reload any open VS Code windows before testing this build:
  Developer: Reload Window

Use the bundled dev binary:
  "yamark.useBundledExecutable": true

Or use a configured Yamark executable:
  "yamark.useBundledExecutable": false
  "yamark.executable": "/path/to/yamark"

For VS Code forks, rerun with CODE_BIN, for example:
  CODE_BIN=codium npm run install:local
EOF

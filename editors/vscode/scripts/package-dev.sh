#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
extension_dir="$(cd "$script_dir/.." && pwd)"
repo_root="$(cd "$extension_dir/../.." && pwd)"

bundle="${YAMARK_BUNDLE:-1}"
profile="${YAMARK_PROFILE:-release}"
out="${VSIX_OUT:-$repo_root/target/vscode/yamark-dev.vsix}"

platform="$(node -p 'process.platform')"
arch="$(node -p 'process.arch')"
exe="yamark"
if [[ "$platform" == "win32" ]]; then
  exe="yamark.exe"
fi

find "$extension_dir/bin" -mindepth 1 -maxdepth 1 -type d -exec rm -rf {} +

if [[ "$bundle" == "1" || "$bundle" == "true" ]]; then
  cargo_args=(build)
  target_dir="$repo_root/target/debug"
  if [[ "$profile" == "release" ]]; then
    cargo_args+=(--release)
    target_dir="$repo_root/target/release"
  elif [[ "$profile" != "debug" ]]; then
    echo "YAMARK_PROFILE must be 'release' or 'debug'" >&2
    exit 2
  fi

  (cd "$repo_root" && cargo "${cargo_args[@]}")

  bundled_dir="$extension_dir/bin/$platform-$arch"
  mkdir -p "$bundled_dir"
  cp "$target_dir/$exe" "$bundled_dir/$exe"
  if [[ "$profile" == "release" ]]; then
    strip "$bundled_dir/$exe"
  fi
  chmod +x "$bundled_dir/$exe"
fi

mkdir -p "$(dirname "$out")"
(cd "$extension_dir" && npx --yes @vscode/vsce package --out "$out")

echo "$out"

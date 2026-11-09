#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/check-markdown.sh [--semantic]

Run Markdown CLI transcripts, public API/specification tests, and format traces.
Requires Rust and Python (used by existing external formatter stubs).
--semantic also runs the existing sentence-wrap Pandoc checks through uv.
Set QUARTO_PANDOC to use a standalone Pandoc; otherwise Quarto is required.

The transcript group is always YAMARK_CASE=markdown. For a smaller case run,
invoke cargo test --test cli_cases with YAMARK_CASE directly.
Run scripts/check.sh for the complete local gate.
EOF
}

semantic=false
case "${1:-}" in
  "") ;;
  --semantic) semantic=true ;;
  -h|--help) usage; exit 0 ;;
  *) usage >&2; exit 2 ;;
esac
if (( $# > 1 )); then
  usage >&2
  exit 2
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

# The harness rejects an empty selection. Override an inherited filter so a
# previous single-case run cannot silently narrow this group's coverage.
YAMARK_CASE=markdown cargo test --test cli_cases -- --nocapture
cargo test --test spec_cli -- \
  markdown_containers:: \
  markdown_directives:: \
  markdown_syntax:: \
  markdown_tables:: \
  markdown_wrapping::
cargo test \
  --test markdown_container_contracts \
  --test markdown_plan \
  --test markdown_preservation \
  --test nested_list_continuation \
  --test sentence_wrap \
  --test source_spans

if "$semantic"; then
  uv run external-tests/run.py --suite cli/test_sentence_wrap.py --serial
fi

# Keep the feature-specific build last; all commands use the binary belonging
# to their Cargo invocation and no independent builds run concurrently.
cargo test --features format-trace --test format_trace

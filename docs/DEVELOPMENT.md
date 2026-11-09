# Development

Run commands from the repository root. Read [AGENTS.md](../AGENTS.md) for change discipline and [Architecture](ARCHITECTURE.md) for implementation ownership.

## Setup

Building the binary requires the Rust version in `Cargo.toml` or newer, with rustfmt and Clippy installed. A local build uses:

```sh
cargo build --bin yamark
```

The complete checks also need Git, uv, Python, Node, Quarto, and R. Install the R packages `stringi`, `jsonlite`, `knitr`, and `yaml12`; CI uses the Posit R-universe repository for `yaml12`. Python test dependencies are supplied by uv in the commands below. See [CI](../.github/workflows/ci.yml) for the tested versions and setup commands.

Build a local Python wheel with `uvx maturin build --release`. The Python package ships the native executable; it does not introduce another formatting engine.

## Validation

For behavior changes, add a failing [CLI transcript](../tests/cases/README.md) before changing production code. For a refactor, run the existing public tests first. Keep the test's assertions and contract intact when moving it.

| Command                                                                                 | Purpose                                                    |
| --------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| `cargo test --test cli_cases`                                                           | Exact CLI transcripts, required after case changes.        |
| `uv run --no-project --with ruff --with py-yaml12 -- cargo test --test spec_cli FILTER` | Focused public CLI specifications.                         |
| `cargo test --test TEST_TARGET FILTER`                                                  | A focused Rust integration suite.                          |
| `scripts/check-markdown.sh`                                                             | Markdown transcripts, API/specification tests, and traces. |
| `scripts/check-markdown.sh --semantic`                                                  | Add the sentence-wrap Pandoc semantic checks.              |
| `uv run external-tests/run.py --suite cli --serial`                                     | External public CLI and semantic checks.                   |
| `npm --prefix editors/vscode test`                                                      | Editor integration tests.                                  |
| `cargo fmt`                                                                             | Format Rust source and tests.                              |
| `git diff --check`                                                                      | Check whitespace in the patch.                             |
| `scripts/check.sh`                                                                      | Complete local gate, also used by CI.                      |

For a focused transcript run, use `YAMARK_CASE=yaml_multiline_ cargo test --test cli_cases -- --nocapture`. `YAMARK_CASE` selects a literal filename substring and fails when no case matches; see [CLI case files](../tests/cases/README.md). Leave it unset for the complete validation gate.

For Markdown parser work, use `scripts/check-markdown.sh` between edits. It runs all transcripts whose filenames contain `markdown`, the Markdown CLI specifications, container/source/plan/preservation/wrapping API suites, and the feature-specific traces. It deliberately overrides an inherited `YAMARK_CASE` so an earlier single-case command cannot narrow the group silently. For a smaller loop, select a case directly, for example `YAMARK_CASE=markdown_list_ cargo test --test cli_cases -- --nocapture`.

The Markdown check needs Rust and Python; existing formatter integration stubs invoke `python`. The default check does not install or upgrade tools. Its optional `--semantic` stage uses the existing uv runner and sentence-wrap Pandoc tests. Supply `QUARTO_PANDOC` for a standalone binary or use an installed Quarto. The default check omits the external semantic suite, allocation/performance checks, editor tests, and packaging/platform coverage; run `scripts/check.sh` before handing off a completed change.

`scripts/check.sh` checks Rust formatting, runs Clippy with all features and targets, runs the Rust tests with Python dependencies, executes the `format-trace` feature's diagnostic tests, runs the external suite, and runs the editor tests. It stops at the first failed command. Read the output and fix the cause before rerunning. Report a failed or unavailable stage rather than counting it as coverage.

Keep the checkout and Git HEAD unchanged during validation; benchmark tests compare artifacts keyed by commit. Use one Cargo build/test owner per checkout. Cargo's build lock does not cover the entire lifetime of integration tests that launch `target/debug/yamark`. Another build can replace that executable between invocations. Parallel test processes within one test run are supported; independent build/test runs need separate checkouts and target directories.

The optional YAML Test Suite round-trip tests run when `tests/yaml-test-suite/data` exists. Prepare it from a local source with:

```sh
tools/bootstrap-yaml-test-suite-data.py --source ~/github/posit-dev/r-yaml12/tests/testthat/yaml-test-suite
```

See [Tests](../tests/README.md) for the difference between transcripts, public API checks, semantic comparisons, and allocation/performance tests.

## Windows

CI runs the complete checks on macOS and Windows, plus the Git path tests on Linux to cover non-UTF-8 filenames in the working tree. On Windows, put `Rscript.exe` and Quarto on `PATH` and set `PYTHONUTF8=1`. Run `scripts/check.sh` from Git Bash, as CI does, or run its commands explicitly from PowerShell. Unix shell-script tests run only on Unix; symlink tests need Developer Mode or symlink privileges.

Keep byte-sensitive fixtures under the existing `.gitattributes` rules. CRLF, bare CR, and mixed line endings are behavioral test data, not formatting noise.

## Documentation

Edit `website/*.qmd` for user-facing contracts. Contributor guides live in `docs/`; historical experiments live in `design-sketches/`. Do not edit rendered `website/*.html.md` as the source of a contract. The Pages workflow owns website rendering.

Format changed prose with `yamark format --wrap paragraph PATH`. Inspect the diff, especially code examples and byte-sensitive fixtures. Keep pending release notes in `RELEASE_NOTES.md` as described in [Release](../RELEASE.md).

# Yamark

Yamark formats YAML, Markdown, Markdown files with YAML front matter,
and explicit embedded Markdown targets in Python and R source files. It
rewrites supported regions and preserves unsupported input. Pass
`--verify` to reparse changed YAML and reject invalid or value-changing
output.

## Install

Run Yamark from PyPI without installing it:

```sh
uvx yamark format config.yaml docs/
```

Install the command from PyPI:

```sh
uv tool install yamark
```

Build and install the binary from a checkout:

```sh
cargo build --bin yamark
cargo install --path .
```

Build a Python wheel from a checkout for local testing:

```sh
uvx maturin build --release
```

## Usage

Format one or more files or directories in place:

```sh
yamark format config.yaml docs/
```

Format the current directory:

```sh
yamark format
```

Check whether files are already formatted without writing changes:

```sh
yamark format --check docs/
```

Show a unified diff without writing changes:

```sh
yamark format --diff docs/
```

Format stdin for editor and CI integrations:

```sh
yamark format --stdin-file-path config.yaml < config.yaml
```

Directory traversal respects `.gitignore`, `.ignore`, and global Git
ignore files by default.

## Development

Run the Rust test suite with:

```sh
cargo test
```

Run formatting and lint checks with:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

Run the external CLI test suite with:

```sh
uv run external-tests/run.py
```

Run the VS Code extension tests with:

```sh
cd editors/vscode
npm test
```

The YAML test-suite roundtrip integration test runs automatically when
`tests/yaml-test-suite/data` exists. Populate that fixture directory
with:

```sh
tools/bootstrap-yaml-test-suite-data.py --source ~/github/posit-dev/r-yaml12/tests/testthat/yaml-test-suite
```

Use `website/reference.qmd` and the public CLI tests as behavior
references.

## Release

Update the package versions in `Cargo.toml`, `pyproject.toml`, and
`editors/vscode/package.json`, refresh `Cargo.lock` and `uv.lock`, then
push a matching `vX.Y.Z` tag:

```sh
git tag v0.2.0
git push origin v0.2.0
```

The release workflow validates the tag, builds binary archives and
Python distributions, smoke-tests each wheel, creates the GitHub release
with generated release notes, and publishes the Python distributions to
PyPI.

## Editor Integrations

The VS Code and Positron formatter extension lives in `editors/vscode/`;
see `editors/vscode/README.md` for install and local development
instructions.

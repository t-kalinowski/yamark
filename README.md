# Yamark

Yamark is a fast formatter for YAML and Markdown. It formats whole files
and embedded content, keeping source readable and changes easy to review.
Regions without a supported rewrite stay unchanged.

For supported embedded code, Yamark can also call Ruff, Air, Prettier,
or another configured formatter.

See the [documentation](https://t-kalinowski.github.io/yamark/) for
examples, configuration, editor integrations, and the full syntax
reference.

## Install

With [uv](https://docs.astral.sh/uv/) installed, run Yamark directly
from [PyPI](https://pypi.org/project/yamark/) without a separate
install:

```sh
uvx yamark format config.yaml docs/
```

This formats the selected files in place. To install a persistent
`yamark` command:

```sh
uv tool install yamark
```

## Usage

The examples below use an installed `yamark` command. To keep running
from PyPI without installing it, replace `yamark` with `uvx yamark`.

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

Directory traversal skips hidden paths and respects `.gitignore`,
`.ignore`, and global Git ignore files by default. Pass a hidden path
explicitly to format it.

## Editor integrations

The VS Code and Positron formatter extension lives in `editors/vscode/`.
See the [editor guide][editor-docs] for installation and configuration.

## Development

Build or install the binary from a checkout with Rust 1.88 or newer:

```sh
cargo build --bin yamark
cargo install --path .
```

Build a Python wheel for local testing:

```sh
uvx maturin build --release
```

Run the Rust tests, formatting check, and lints:

```sh
cargo test
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

Run the external CLI and VS Code extension tests separately:

```sh
uv run external-tests/run.py
cd editors/vscode
npm test
```

When `tests/yaml-test-suite/data` exists, `cargo test` also runs the
YAML Test Suite round-trip test. Populate that directory with:

```sh
tools/bootstrap-yaml-test-suite-data.py --source ~/github/posit-dev/r-yaml12/tests/testthat/yaml-test-suite
```

Use the pages linked from `website/reference.qmd` and the public CLI tests as
behavior references.

## Release

Update the package versions in `Cargo.toml`, `pyproject.toml`, and
`editors/vscode/package.json`, refresh `Cargo.lock` and `uv.lock`, then
push a matching `vX.Y.Z` tag:

```sh
git tag vX.Y.Z
git push origin vX.Y.Z
```

The release workflow validates the tag, builds binary archives and
Python distributions, smoke-tests each wheel, creates the GitHub release
with generated release notes, and publishes the Python distributions to
PyPI.

[editor-docs]: https://t-kalinowski.github.io/yamark/editors.html

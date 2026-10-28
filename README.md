# Yamark

Yamark is a fast formatter for YAML and Markdown. It formats whole files and embedded content, keeping source readable and changes easy to review. Regions without a supported rewrite stay unchanged.

For supported embedded code, Yamark can also call Ruff, Air, Prettier, or another configured formatter.

See the [documentation](https://t-kalinowski.github.io/yamark/) for examples, configuration, editor integrations, and the full syntax reference.

## Install

With [uv](https://docs.astral.sh/uv/) installed, run Yamark directly from [PyPI](https://pypi.org/project/yamark/) without a separate install:

```sh
uvx yamark format config.yaml docs/
```

This formats the selected files in place. To install a persistent `yamark` command:

```sh
uv tool install yamark
```

## Usage

The examples below use an installed `yamark` command. To keep running from PyPI without installing it, replace `yamark` with `uvx yamark`.

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

Convert JSON-family input from stdin to formatted YAML on stdout:

```sh
yamark to-yaml --stdin-file-path data.json5 < data.json5
```

The path is not read or written. Its suffix selects the exact JSON dialect. This is a conversion, not source formatting.

Directory traversal skips hidden paths and respects `.gitignore`, `.ignore`, and global Git ignore files by default. Pass a hidden path explicitly to format it.

While formatting is enabled, standalone `{{< ... >}}` and `{{% ... %}}` shortcode tokens are preserved, including multiline tokens and quoted arguments. Directive-looking text inside a recognized token is data. Yamark does not match shortcode names or interpret their bodies: Markdown between tokens formats normally. An unterminated token or quote preserves the rest of the enclosing Markdown document or fragment. This does not guarantee rendering equivalence for arbitrary Hugo or Jinja templates.

To disable body formatting, surround the region with `<!-- fmt: off -->` and `<!-- fmt: on -->`. While formatting is disabled, directives follow the existing linewise policy: a recognized `fmt: on` line resumes formatting even if the surrounding text resembles a shortcode argument, code fence, HTML, or math block.

Explicitly skipped Markdown retains its bytes, including trailing spaces and line endings: `<!-- fmt: skip -->` preserves the next selected node, `<!-- fmt: off -->` preserves the disabled region, and `<!-- fmt: skip file -->` preserves the enclosing Markdown document or fragment. This protection survives supported nesting in Markdown fences and divs; surrounding Markdown still formats.

## Editor integrations

The VS Code and Positron formatter extension lives in `editors/vscode/`. See the [editor guide][editor-docs] for installation and configuration.

## Development

See [Development](docs/DEVELOPMENT.md) for setup and validation, [Architecture](docs/ARCHITECTURE.md) for implementation ownership, and the [test index](tests/README.md) for focused suites. Contributor rules are in [AGENTS.md](AGENTS.md).

Building from source requires Rust 1.98.1 or newer.

Build or install the binary from a checkout:

```sh
cargo build --bin yamark
cargo install --path .
```

Build a Python wheel for local testing:

```sh
uvx maturin build --release
```

The full test suite also uses `uv`, Python, Ruff, Quarto, Node, and R with `stringi`, `jsonlite`, `knitr`, and `yaml12` installed. Run the Rust tests with Python dependencies available, followed by the formatting check and lints:

```sh
uv run --no-project --with ruff --with py-yaml12 -- cargo test
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

Run the external CLI and VS Code extension tests separately:

```sh
uv run external-tests/run.py
cd editors/vscode
npm test
```

CI runs these checks on macOS and Windows. On Windows, put `Rscript.exe` and Quarto on `PATH` and set `$env:PYTHONUTF8 = "1"` in PowerShell before running the tests. Unix shell-script tests run on macOS; symlink tests need Windows Developer Mode or symlink privileges. Git attributes keep source and transcript files at LF while byte-sensitive fixtures retain their original line endings.

When `tests/yaml-test-suite/data` exists, `cargo test` also runs the YAML Test Suite round-trip test. Populate that directory with:

```sh
tools/bootstrap-yaml-test-suite-data.py --source ~/github/posit-dev/r-yaml12/tests/testthat/yaml-test-suite
```

Use the pages linked from `website/reference.qmd` and the public CLI tests as behavior references.

## Release

See the [release guide](https://github.com/t-kalinowski/yamark/blob/main/RELEASE.md) for the stable-version bump, checks, release notes, tag, publication verification, and post-release `+dev` bump.

[editor-docs]: https://t-kalinowski.github.io/yamark/editors.html

---
title: Usage
description: Install Yamark, format files, and choose an integration.
---

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

The examples below use an installed `yamark` command. To keep running
from PyPI without installing it, replace `yamark` with `uvx yamark`.

To build from a checkout, install Rust 1.88 or newer and run:

```sh
cargo install --path .
```

## Format files

Format one or more files or directories in place:

```sh
yamark format config.yaml docs/
```

Format the current directory:

```sh
yamark format
```

By default, Markdown prose wraps at column 72 without forcing sentence
boundaries. Override this behavior with `--wrap`, for example:

```sh
yamark format --wrap sentence:88 docs/
```

Directory traversal skips hidden paths and respects `.gitignore`,
`.ignore`, and global Git ignore files by default. Pass a hidden path
explicitly to format it.

## CI and stdin

Use check, diff, or stdin modes for integrations:

```sh
yamark format --check docs/
yamark format --diff docs/
yamark format --stdin-file-path config.yaml < config.yaml
```

`--check` and `--diff` do not write files. Both exit `1` when any
selected file would change.

Render JSON-family input as formatted YAML without changing the source:

```sh
yamark render --stdin-file-path data.json5 < data.json5
```

Use `--diagnostics` to explain preserved content. Use
`--skip-embedded-formatters` when another tool formats the same embedded code.

## Integrations

- [Examples](examples.qmd): focused before-and-after output from the current
  Yamark binary.
- [Directives](directives.qmd): put formatting choices beside content in
  Markdown, YAML, Python, and R files.
- [Editors](editors.qmd): VS Code, Positron, and compatible forks -
  commands, settings, format-on-save, formatter chaining, and logs.
- [Reference](reference.qmd): look up supported files and syntax, formatting
  settings, `yamark.toml`, directive grammar, and command behavior.
- [Command line](cli-help.qmd): modes, output, exit status, and generated
  `--help` for Yamark, `format`, `render`, and the `git-filter` command group.
- [Git Filter](git-filter.qmd): an experimental workflow that stores Markdown
  sentence-per-line while keeping the working tree column-wrapped.

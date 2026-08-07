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

Use `--diagnostics` for preservation notes, or
`--skip-embedded-formatters` when another formatter owns source-code
chunks in the same save or CI chain.

Pass `--verify` to reparse changed YAML and reject invalid or
value-changing output before writing.

## Integrations

- [Editors](editors.qmd): VS Code, Positron, and compatible forks -
  commands, settings, format-on-save, formatter chaining, and logs.
- [Git Filter](git-filter.qmd): store Markdown sentence-per-line in Git
  while keeping the working tree column-wrapped.
- [Reference](reference.qmd): the full CLI option list, `yamark.toml`
  schema, directive syntax, and supported syntax coverage.
- [CLI Help](cli-help.qmd): rendered `yamark --help`,
  `yamark format --help`, and `yamark git-filter --help` output.

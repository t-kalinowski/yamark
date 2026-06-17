---
title: Usage
description: Install yamark, run the formatter, and choose an integration.
---

## Install

Build the `yamark` binary from a checkout:

```sh
cargo build --bin yamark
```

Run the debug binary directly:

```sh
target/debug/yamark format config.yaml docs/
```

Install the binary from the checkout:

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

Directory traversal respects `.gitignore`, `.ignore`, and global Git ignore
files by default.

## CI and stdin

Use check, diff, or stdin modes for integrations:

```sh
yamark format --check docs/
yamark format --diff docs/
yamark format --stdin-file-path config.yaml < config.yaml
```

`--check` and `--diff` do not write files. Both exit `1` when any selected file
would change.

Use `--diagnostics` for preservation notes, or
`--skip-embedded-formatters` when another formatter owns source-code chunks in
the same save or CI chain.

## Integrations

- [Editors](editors.qmd): VS Code, Positron, and compatible forks - commands,
  settings, format-on-save, formatter chaining, and logs.
- [Git Filter](git-filter.qmd): store Markdown sentence-per-line in Git while
  keeping the working tree column-wrapped.
- [Reference](reference.qmd): the full CLI option list, `yamark.toml` schema,
  directive syntax, and supported syntax coverage.
- [CLI Help](cli-help.qmd): rendered `yamark --help`, `yamark format --help`,
  and `yamark git-filter --help` output.

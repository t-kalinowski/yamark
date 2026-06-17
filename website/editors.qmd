---
title: Editors
description: Use Yamark from VS Code, Positron, and compatible forks.
---

The repository ships a Yamark VS Code extension under `editors/vscode`. It runs
`yamark format --stdin-file-path <file>` through the public VS Code formatter
API, so the same extension works in Positron and compatible forks such as
Codium and Cursor.

## Commands

The extension contributes:

| Command | Effect |
| --- | --- |
| `Yamark: Format Document` | Run the Yamark document formatter for the active file. |
| `Yamark: Format Selection as Markdown` | Format the active non-empty selection as Markdown only. |
| `Yamark: Show Log` | Open the Yamark output channel. |

`Yamark: Format Selection as Markdown` does not run configured native formatter
chains. It is for prompt text, comments, or Markdown-like prose inside a
broader source file.

## Executable

Install `yamark` on `PATH`, or set an explicit executable:

```json
{
  "yamark.executable": "/path/to/yamark"
}
```

The extension can also use a bundled executable under
`bin/<platform>-<arch>/` when a VSIX package includes one. Enable that path
with:

```json
{
  "yamark.useBundledExecutable": true
}
```

`yamark.useBundledExecutable` defaults to `false`, so the extension uses
`yamark.executable` unless configured otherwise.

## File extensions

By default Yamark is enabled for Markdown, Quarto, and YAML:

```json
{
  "yamark.enabledFileExtensions": [".md", ".qmd", ".yaml", ".yml"]
}
```

Opt into R Markdown, R, and Python by adding extensions:

```json
{
  "yamark.enabledFileExtensions": [
    ".md",
    ".qmd",
    ".yaml",
    ".yml",
    ".rmd",
    ".r",
    ".py"
  ]
}
```

For `.r` and `.py`, Yamark formats explicitly marked embedded Markdown comment
blocks and string literals. It does not format surrounding source code unless
you configure a second formatter.

## Format on save

Set Yamark as the default formatter for each language it should handle:

```json
{
  "yamark.useBundledExecutable": false,
  "[markdown]": {
    "editor.defaultFormatter": "yamark.yamark",
    "editor.formatOnSave": true
  },
  "[yaml]": {
    "editor.defaultFormatter": "yamark.yamark",
    "editor.formatOnSave": true
  },
  "[quarto]": {
    "editor.defaultFormatter": "yamark.yamark",
    "editor.formatOnSave": true
  }
}
```

Manual `Format Document` and save-time formatting both go through the Yamark
provider.

## Extra arguments

Pass CLI formatter options through `yamark.extraArguments`:

```json
{
  "yamark.extraArguments": ["--wrap", "sentence"]
}
```

Arguments are inserted after `yamark format` and before
`--stdin-file-path <file>`.

## Formatter chaining

Yamark can run one stdin/stdout formatter after itself. This is useful when
Yamark formats embedded Markdown in a Python or R file and Ruff or Air should
then format the surrounding source.

The chain is fixed:

1. Yamark runs first.
2. If `yamark.runNextFormatter` is true and
   `yamark.nextFormatterExecutable` is configured, that executable runs on
   Yamark's output.
3. The extension returns one combined edit to VS Code.

`yamark.runNextFormatter` defaults to `true`. With no
`yamark.nextFormatterExecutable`, Yamark runs alone.

```json
{
  "yamark.enabledFileExtensions": [".md", ".qmd", ".yaml", ".yml", ".r", ".py"],
  "[r]": {
    "editor.defaultFormatter": "yamark.yamark",
    "editor.formatOnSave": true,
    "yamark.nextFormatterExecutable": [
      "${extension:posit.air-vscode}/bundled/bin/air${exe}",
      "format",
      "--stdin-file-path",
      "${file}"
    ]
  },
  "[python]": {
    "editor.defaultFormatter": "yamark.yamark",
    "editor.formatOnSave": true,
    "yamark.nextFormatterExecutable": [
      "${extension:charliermarsh.ruff}/bundled/libs/bin/ruff${exe}",
      "format",
      "--stdin-filename",
      "${file}",
      "-"
    ]
  }
}
```

`yamark.nextFormatterExecutable` is an argv array, not a shell string. Put each
executable and argument in its own array element.

Placeholders:

| Placeholder | Expands to |
| --- | --- |
| `${file}` | Absolute path to the active file. |
| `${fileDirname}` | Absolute directory containing the active file. |
| `${exe}` | `.exe` on Windows, empty string elsewhere. |
| `${extension:publisher.name}` | Install path for a VS Code extension, for example `${extension:posit.air-vscode}`. |

Yamark does not chain through VS Code formatter commands or provider
fallthrough because VS Code does not expose a stable API for running another
formatter second and returning one final save-time edit.

## Logs

Open `View -> Output -> Yamark`, or run `Yamark: Show Log`. The log records the
file path, language id, Yamark invocation, optional follow-up formatter
invocation, edit counts, and byte counts before and after formatting.

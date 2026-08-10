---
title: Editors
description: Use Yamark from VS Code, Positron, and compatible forks.
---

The repository includes a Yamark extension for VS Code, Positron, and compatible
forks. It runs `yamark format --stdin-file-path <file>` for document formatting
and native formatted previews. It runs
`yamark to-yaml --stdin-file-path <file>` for JSON-to-YAML views.

## Install

The extension is not yet published to an extension marketplace. Install the
Yamark command from PyPI, then build and install the extension from a checkout:

```sh
uv tool install yamark
cd editors/vscode
YAMARK_BUNDLE=0 npm run install:local
```

For Positron, use `YAMARK_BUNDLE=0 npm run install:positron`. Reload the editor
after installation.

## Commands

The extension contributes:

| Command | Effect |
| --- | --- |
| `Yamark: Format Document` | Run the Yamark document formatter for the active file. |
| `Yamark: Format Selection as Markdown` | Format the active non-empty selection as Markdown only. |
| `Yamark: Preview Format Document` | Run the same formatting stages as Format Document and open the result as a read-only snapshot. Supports Markdown, R Markdown, Quarto, YAML, Python, and R. |
| `Yamark: View JSON as YAML` | Project JSON, JSONC, JSON5, JSONL, or NDJSON to formatted YAML in a read-only view. |
| `Yamark: Open Filtered Working Tree Diff` | Compare an unstaged file with its smudged Git index baseline. |
| `Yamark: Show Log` | Open the Yamark output channel. |

`Yamark: Format Selection as Markdown` does not run configured native formatter
chains. It is for prompt text, comments, or Markdown-like prose inside a
broader source file.

## Read-only previews

Right-click a supported file in the Explorer or its editor tab. Use
`Yamark: Preview Format Document` for Yamark's native file types, or
`Yamark: View JSON as YAML` for JSON, JSONC, JSON5, JSONL, and NDJSON.

`Yamark: Preview Format Document` runs the same formatting stages as Format
Document, including the configured next formatter, but opens the final text
instead of applying an edit. `Yamark: View JSON as YAML` sends the current
editor buffer to `yamark to-yaml` and opens the resulting YAML. Both commands
include unsaved source edits, do not change the source or create a temporary
file, and refresh only when run again.

JSON-family previews use YAML syntax highlighting. JSONC and JSON5 comments
become YAML comments. Each JSONL or NDJSON record becomes one document in a
YAML stream.

Preview commands do not use `yamark.enabledFileExtensions`. The JSON-to-YAML
projection does not run the configured next formatter, apply
`yamark.extraArguments`, or interpret embedded formatter directives.
JSON-family files remain excluded from Format Document and format-on-save.

## Git filter diffs

For a path managed by `filter=yamark-md`, VS Code's built-in unstaged preview
can show wrapping changes that are not part of the Git diff. Right-click the
file under **Changes** and run `Yamark: Open Filtered Working Tree Diff`.

See [Git Filter: VS Code diff preview](git-filter.qmd#vs-code-diff-preview) for
the clean/smudge details and the limits of this command.

## Executable

The extension runs `yamark` from `PATH` by default. To use another binary, set
an explicit executable:

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
`--stdin-file-path <file>`. They also apply to Preview Format Document because
that command uses the same formatting pipeline.

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

Open `View -> Output -> Yamark`, or run `Yamark: Show Log`. Formatting and
Preview Format Document logs record the file, Yamark invocation, optional
follow-up formatter, changes, and errors. View JSON as YAML logs record the
source, `to-yaml` invocation, output size, and errors.

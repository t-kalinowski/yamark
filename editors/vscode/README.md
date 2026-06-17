# Yamark for VS Code and Positron

This extension formats files by running:

```sh
yamark format --stdin-file-path <file>
```

It uses the public VS Code extension API, so it also works in Positron
and compatible VS Code forks.

## Requirements

Install `yamark` and make it available on `PATH`, or set:

```json
{
  "yamark.executable": "/path/to/yamark"
}
```

The extension can also use a bundled executable when a VSIX package includes
one. Package platform-specific Yamark binaries under `bin/<platform>-<arch>/`
and enable:

```json
{
  "yamark.useBundledExecutable": true
}
```

`yamark.useBundledExecutable` defaults to `false`, so a public install uses the
`yamark` executable from `PATH` unless configured otherwise.

## Local Development Install

Build a VSIX with the current local Yamark binary bundled:

```sh
npm run build:dev
```

Install into VS Code:

```sh
npm run install:local
```

Install into Positron:

```sh
npm run install:positron
```

Use `CODE_BIN` for a custom Positron path or another VS Code-compatible
fork:

```sh
CODE_BIN=/path/to/positron npm run install:local
CODE_BIN=codium npm run install:local
CODE_BIN=cursor npm run install:local
```

Without a command line launcher, run `npm run build:dev`, then use
`Extensions: Install from VSIX...` and select
`target/vscode/yamark-dev.vsix`.

The dev package builds the local `yamark` package and bundles
`target/release/yamark` by default. Use `YAMARK_PROFILE=debug` to bundle
the debug build, or `YAMARK_BUNDLE=0` to build the extension package
without copying a binary.

## File Extensions

Open `Preferences: Open User Settings (JSON)` in VS Code. The examples
below are complete `settings.json` files. If your settings file already
has entries, copy the keys inside the outer `{ ... }` into your existing
top-level object.

By default Yamark is enabled for Markdown, Quarto, and YAML file extensions:

```json
{
  "yamark.enabledFileExtensions": [".md", ".qmd", ".yaml", ".yml"]
}
```

Opt into R Markdown, R, or Python files by adding extensions:

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

For R and Python, Yamark formats explicitly marked embedded Markdown
strings inside the source file. It can also run a language formatter
after Yamark when that formatter exposes a stdin/stdout CLI. See
"Composing With Native Formatters" below.

## Yamark Config For Embedded Formatters

Yamark can also format explicitly marked embedded targets through
`yamark.toml`. In this repository, embedded formatter entries use
`[embedded.<name>]` with a `formatter = ...` value:

```toml
[embedded.python]
formatter = "ruff"

[embedded.r]
formatter = "air"

[embedded.custom]
formatter = { command = ["tool", "--stdin-file-path", "{path}"], path_suffix = ".ext" }
```

Do not use the old nested `[embedded.<name>.formatter]` table shape.
This config belongs to Yamark itself; VS Code save-time composition with
another formatter uses `yamark.nextFormatterExecutable` as described
below.

## Format And Save Behavior

Yamark registers as a document formatting provider for the configured
file extensions. To use it, set `editor.defaultFormatter` to
`yamark.yamark` for each language Yamark should handle, and enable
`editor.formatOnSave` either globally or per language. Both manual
`Format Document` (Cmd+Shift+I / Ctrl+Shift+I) and save-time formatting
go through Yamark's provider.

When Yamark also needs to compose with a stdin/stdout language
formatter, see the next section.

## Format Selection As Markdown

Run `Yamark: Format Selection as Markdown` from the Command Palette to
format only the active selection as Markdown. This command is useful for
prompt text or Markdown-like prose inside source files where formatting
the whole document would be too broad.

The command requires a non-empty selection. With an empty selection,
Yamark leaves the document unchanged and reports `Yamark: no text
selected.` in the status bar. The selected text is formatted as Markdown
only; configured native formatter chains are not run for this command.

## Composing With Native Formatters

Yamark only formats Markdown prose, YAML frontmatter, and embedded
Markdown string literals inside source files. Each language has its own
formatter for the surrounding code (Ruff for Python, Air for R, the
Quarto extension's formatter for Quarto, rust-analyzer or rustfmt for
Rust, etc.). Yamark can compose with those formatters only when they
expose a stdin/stdout executable.

### How The Chain Works

Yamark supports one save-stable chaining shape:

1. `yamark format --stdin-file-path` runs first.
2. If `yamark.runNextFormatter` is true and
   `yamark.nextFormatterExecutable` is configured, Yamark runs that
   stdin/stdout executable on Yamark's output.
3. Yamark's provider returns one combined `TextEdit` for VS Code to
   apply.

This preserves the intended order: Yamark first, then the native
formatter. VS Code does not expose a stable provider API that lets
Yamark select another formatter provider, run it after Yamark, and still
return one final edit during format-on-save. Yamark therefore does not
chain through VS Code formatter commands or provider fallthrough.

Use a stdin/stdout executable when you need a second formatter:

```jsonc
"[r]": {
  "editor.defaultFormatter": "yamark.yamark",
  "editor.formatOnSave": true,
  "editor.insertSpaces": true,
  "editor.tabSize": 2,
  "yamark.nextFormatterExecutable": [
    "${extension:posit.air-vscode}/bundled/bin/air${exe}",
    "format",
    "--stdin-file-path",
    "${file}"
  ]
}
```

The chain is enabled by default through:

- `yamark.runNextFormatter` — run the configured executable after
  Yamark (default `true`). With no executable configured, Yamark runs by
  itself.
- `yamark.nextFormatterExecutable` — optional stdin/stdout formatter
  argv. The first array element is the executable and the rest are
  arguments.

Placeholders in `yamark.nextFormatterExecutable` are expanded inside
each argv element before the process starts:

| Placeholder | Expands To |
| --- | --- |
| `${file}` | Absolute path to the document being formatted. |
| `${fileDirname}` | Absolute directory path containing the document. |
| `${exe}` | `.exe` on Windows, empty string on other platforms. |
| `${extension:publisher.name}` | Install path for a VS Code extension, for example `${extension:posit.air-vscode}`. |

The setting is an argv array, not a shell string. Put each executable
and argument in its own array element; shell quoting and shell
expansion are not applied.

### Uniform Per-Language Shape

Use the same shape inside `[<lang>]` for every language Yamark touches:

```json
"[<lang>]": {
  "editor.defaultFormatter": "yamark.yamark",
  "editor.formatOnSave": true
}
```

Omit `yamark.nextFormatterExecutable` when Yamark should be the only
formatter in that language. Set `yamark.nextFormatterExecutable` when
you need to pick a specific stdin/stdout formatter, such as Air or Ruff.

```jsonc
"[r]": {
  "editor.defaultFormatter": "yamark.yamark",
  "editor.formatOnSave": true,
  "yamark.nextFormatterExecutable": [
    "${extension:posit.air-vscode}/bundled/bin/air${exe}",
    "format",
    "--stdin-file-path",
    "${file}"
  ]
}
```

Ruff's extension bundles a stdin/stdout formatter executable:

```jsonc
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
```

Disable the chain for a specific language (rarely needed) with
`"yamark.runNextFormatter": false` inside the language block.

### Common Setup

This complete `settings.json` enables Yamark for Markdown, YAML, Quarto,
R Markdown, R, and Python. The R and Python blocks show executable
chains; the other language blocks run Yamark only.

```jsonc
{
  "yamark.useBundledExecutable": false,
  "yamark.enabledFileExtensions": [
    ".md",
    ".qmd",
    ".yaml",
    ".yml",
    ".rmd",
    ".r",
    ".py"
  ],

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
    "editor.formatOnSave": true,
    "editor.insertSpaces": true,
    "editor.tabSize": 2
  },
  "[rmd]": {
    "editor.defaultFormatter": "yamark.yamark",
    "editor.formatOnSave": true,
    "editor.insertSpaces": true,
    "editor.tabSize": 2
  },
  "[r]": {
    "editor.defaultFormatter": "yamark.yamark",
    "editor.formatOnSave": true,
    "editor.insertSpaces": true,
    "editor.tabSize": 2,
    // Chain: Yamark -> Air executable.
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
    // Chain: Yamark -> Ruff executable.
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

With `yamark.nextFormatterExecutable`, Yamark runs first and then runs
that argv on Yamark's output.

The Quarto extension formatter is not configured as a second formatter
in this setup because it is not exposed as a stdin/stdout executable.
Leave `yamark.nextFormatterExecutable` unset for `[quarto]` and `[rmd]`
unless you intentionally configure a specific stdin/stdout executable.

The common R Markdown language id is `rmd`. If VS Code shows a
different language id in the status bar for `.Rmd` files, replace
`[rmd]` with that id.

### Adding A New Language

To compose Yamark with a new language's formatter:

1. Add the file extension to `yamark.enabledFileExtensions` (e.g.
   `.rs`).
2. Install the language's native formatter CLI.
3. Add a `[<lang>]` block with `editor.defaultFormatter:
   "yamark.yamark"` and `editor.formatOnSave: true`.
4. If the language formatter has a stdin/stdout CLI, add it as
   `yamark.nextFormatterExecutable` in the same block.

No code change to Yamark is needed.

### Removed Settings

Older development builds exposed `yamark.formatOnSave`,
`yamark.formatThenNextFormatter`, `yamark.nextFormatterCommand`, and
`yamark.nextFormatterCommands`. They are no longer contributed by the
extension, and command-based formatter chaining is ignored.

Use VS Code's `editor.formatOnSave` and `editor.defaultFormatter` for
save behavior. Use `yamark.nextFormatterExecutable` when Yamark should
run a second stdin/stdout formatter after Yamark.

## Inspecting Logs

Yamark writes a structured trace of each format-on-save run to its own
output channel. Open it with `View → Output → Yamark`, or run
`Yamark: Show Log` from the command palette.

Each format operation uses one `[format <id>]` correlation id. Start
with the `document` entry: `uri`, `path`, `languageId`, `version`,
`dirty`, and `trigger` show which document was formatted and whether
the provider or `Yamark: Format Document` command started the run.

Formatter entries use `formatter step=start|end|skip|error` with a
`name` such as `vscode-provider`, `yamark`, `next-executable`, or
`legacy-command`. The `edits`, `applied`, `captured`, `changed`, and
byte-count fields show whether each formatter changed text, whether
Yamark captured those edits for VS Code to apply, and whether a command
run applied them directly. `suppression action=add|hit|remove` entries
mark command-apply windows where reentrant provider calls intentionally
return no edits.

For manual verification, install the development VSIX and check
format-on-save plus `Yamark: Format Document` on `.py`, `.r`, `.R`,
`.qmd`, `.md`, and `.yaml` files.

## Extra Yamark Arguments

Pass formatter options through `yamark.extraArguments`:

```json
{
  "yamark.extraArguments": ["--wrap", "sentence"]
}
```

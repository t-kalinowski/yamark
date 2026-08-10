# Yamark for VS Code and Positron

This extension formats files by running:

```sh
yamark format --stdin-file-path <file>
```

It opens read-only formatted previews by running:

```sh
yamark render --stdin-file-path <file>
```

It uses the public VS Code extension API, so it also works in Positron
and compatible VS Code forks.

## Install

The extension is not yet published to an extension marketplace. Install the
Yamark command from PyPI:

```sh
uv tool install yamark
```

Then install the extension from a checkout without bundling another binary:

```sh
cd editors/vscode
YAMARK_BUNDLE=0 npm run install:local
```

For Positron, use `YAMARK_BUNDLE=0 npm run install:positron`. Reload the editor
after installation. The extension runs `yamark` from `PATH` by default.

To use another executable, set:

```json
{
  "yamark.executable": "/path/to/yamark"
}
```

## Bundled development build

For extension development, build and install a VSIX with the current local
Yamark binary bundled. This requires Rust 1.88 or newer:

```sh
npm run install:local
```

Use `npm run install:positron` for Positron. Then enable the bundled executable:

```json
{
  "yamark.useBundledExecutable": true
}
```

Use `CODE_BIN` for a custom Positron path or another VS Code-compatible
fork:

```sh
CODE_BIN=/path/to/positron npm run install:local
CODE_BIN=codium npm run install:local
CODE_BIN=cursor npm run install:local
```

To build the VSIX without installing it, run `npm run build:dev`. Without a
command-line launcher, use
`Extensions: Install from VSIX...` and select
`target/vscode/yamark-dev.vsix`.

The development package builds and bundles `target/release/yamark` by default.
Use `YAMARK_PROFILE=debug` to bundle the debug build, or `YAMARK_BUNDLE=0` to
build the extension without copying a binary.

## File extensions

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

For R and Python, Yamark formats `#|` hashpipe YAML comment blocks and
explicitly marked targets: Markdown comment blocks and string literals,
or string literals marked for an embedded formatter. It preserves the
surrounding source code. To format the whole source document as well,
configure a follow-up formatter; see "Composing with native formatters"
below.

## Yamark config for embedded formatters

`yamark.toml` can configure the formatter used for an explicit embedded
target. Embedded formatter entries use `[embedded.<name>]` with a
`formatter = ...` value:

```toml
[embedded.python]
formatter = "ruff"

[embedded.r]
formatter = "air"

[embedded.custom]
formatter = { command = ["tool", "--stdin-file-path", "{path}"], path_suffix = ".ext" }
```

This config belongs to Yamark itself; VS Code save-time composition with
another formatter uses `yamark.nextFormatterExecutable` as described
below.

## Format and save behavior

Yamark registers as a document formatting provider for the configured
file extensions. To use it, set `editor.defaultFormatter` to
`yamark.yamark` for each language Yamark should handle, and enable
`editor.formatOnSave` either globally or per language. Both manual
`Format Document` (Cmd+Shift+I / Ctrl+Shift+I) and save-time formatting
go through Yamark's provider.

When Yamark also needs to compose with a stdin/stdout language
formatter, see the next section.

## Format selection as Markdown

Run `Yamark: Format Selection as Markdown` from the Command Palette to
format only the active selection as Markdown. This command is useful for
prompt text or Markdown-like prose inside source files where formatting
the whole document would be too broad.

The command requires a non-empty selection. With an empty selection,
Yamark leaves the document unchanged and reports `Yamark: no text
selected.` in the status bar. The selected text is formatted as Markdown
only; configured native formatter chains are not run for this command.

## Read-only formatted previews

Right-click a supported file in the Explorer or its editor tab. Use
`Yamark: Open Formatted Preview` for Markdown, R Markdown, Quarto, YAML,
Python, and R.
Use `Yamark: View JSON as YAML` for JSON, JSONC, JSON5, JSONL, and NDJSON.

The command sends the current editor buffer to Yamark and opens the result as a
read-only snapshot. It does not change the source or create a temporary file.
Unsaved edits are included. Run the command again to refresh the same preview;
source edits do not refresh it automatically.

JSON-family previews use YAML syntax highlighting. JSONC and JSON5 comments
become YAML comments. Each JSONL or NDJSON record becomes one YAML stream
document.

Preview commands do not use `yamark.enabledFileExtensions` or run
`yamark.nextFormatterExecutable`. JSON-family files remain excluded from
Format Document and format-on-save.

## Git filter diffs

VS Code's built-in unstaged preview compares the clean index blob with the
smudged working-tree file. For files managed by `filter=yamark-md`, wrapping
changes can obscure the content change.

Right-click a modified tracked file under **Changes** and run
`Yamark: Open Filtered Working Tree Diff`. The command reads the index with
Git's checkout filters and compares that smudged baseline with the working-tree
file. Use the normal VS Code diff under **Staged Changes**, where both HEAD and
the index use the clean storage form.

This command does not replace VS Code's default row click or gutter markers.
VS Code does not expose those parts of its Git integration through the public
extension API.

## Composing with native formatters

Embedded target formatting and whole-document chaining have different
scopes. An embedded formatter receives only its target inside a
document. `yamark.nextFormatterExecutable` instead receives Yamark's
full document output, so it can format the surrounding Python or R
source afterward.

The extension can chain one formatter only when it exposes a
stdin/stdout executable. Ruff and Air are examples for Python and R.

### How the chain works

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

### Uniform per-language shape

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

### Common setup

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

### Adding a new language

To compose Yamark with a new language's formatter:

1. Add the file extension to `yamark.enabledFileExtensions` (e.g.
   `.rs`).
2. Install the language's native formatter CLI.
3. Add a `[<lang>]` block with `editor.defaultFormatter:
   "yamark.yamark"` and `editor.formatOnSave: true`.
4. If the language formatter has a stdin/stdout CLI, add it as
   `yamark.nextFormatterExecutable` in the same block.

No code change to Yamark is needed.

## Inspecting logs

Yamark writes a structured trace of each format-on-save run to its own
output channel. Open it with `View → Output → Yamark`, or run
`Yamark: Show Log` from the command palette.

Each format or preview operation has one correlation id. Format logs record the
active document, Yamark arguments, optional follow-up formatter, whether text
changed, and any error. Preview logs record the source document, render
arguments, output size, and any error.

For manual verification, install the development VSIX and check
format-on-save plus `Yamark: Format Document` on `.py`, `.r`, `.R`,
`.qmd`, `.md`, and `.yaml` files.

## Extra Yamark arguments

Pass formatter options through `yamark.extraArguments`:

```json
{
  "yamark.extraArguments": ["--wrap", "sentence"]
}
```

The extension inserts these arguments after `yamark format` or `yamark render`
and before `--stdin-file-path <file>`.

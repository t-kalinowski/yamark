---
title: Command line
description: Modes, output, exit status, and generated command-line help.
execute:
  echo: false
  warning: false
  message: false
  comment: ""
---



Use [Usage](usage.qmd) for common commands and
[Formatting settings](reference-options.qmd) to find a control by the output it
changes. This command-line interface (CLI) reference includes help screens
generated from the current Yamark binary.

## Modes, output, and status

| Mode | Files | stdout | stderr |
| --- | --- | --- | --- |
| `yamark format PATHS` | Writes changed files. | Summary. | Failures and requested diagnostics. |
| `yamark format --check PATHS` | Does not write. | - | Summary, failures, and requested diagnostics. |
| `yamark format --diff PATHS` | Does not write. | Unified diffs. | Summary, failures, and requested diagnostics. |
| `yamark format --stdin-file-path PATH` | Does not read or write `PATH`; uses it to select file-aware behavior for stdin. | Formatted content only. | Failures and requested diagnostics. |
| `yamark render --stdin-file-path PATH` | Does not read or write `PATH`; renders stdin using its native or JSON-family file type. | Read-only formatted content only. | Failures. |

`--check` and `--diff` exit `1` when any selected file would change. Every mode
exits `1` when formatting fails and `0` on success otherwise. Invalid command
syntax exits `2`.

When `PATHS` is omitted from `yamark format`, Yamark uses the current directory
(`.`). Format path mode counts unsupported extensions as skipped without
failing. An unsupported `--stdin-file-path` is an error because stdin mode
requires a supported file type.

Format stdin mode rejects additional paths and cannot be combined with
`--check` or `--diff`. `yamark render` always reads stdin, requires
`--stdin-file-path`, and never writes a file.

## Generated help

### `yamark`

`````{=html}
<pre class="yamark-cli-help"><code>A fast formatter for YAML and Markdown.

Run `yamark &lt;COMMAND&gt; --help` for command-level help.

<span style='color: #5555FF; font-weight: bold;'>Usage:</span> <span style='color: #00BBBB; font-weight: bold;'>yamark</span> <span style='color: #555555;'>&lt;COMMAND&gt;</span>

<span style='color: #5555FF; font-weight: bold;'>Commands:</span>
  <span style='color: #00BBBB; font-weight: bold;'>format</span>
  <span style='color: #00BBBB; font-weight: bold;'>render</span>      Render a read-only formatted view from stdin
  <span style='color: #00BBBB; font-weight: bold;'>git-filter</span>  Git clean/smudge filter helpers for Markdown files
  <span style='color: #00BBBB; font-weight: bold;'>help</span>        Print this message or the help of the given subcommand(s)

<span style='color: #5555FF; font-weight: bold;'>Options:</span>
  <span style='color: #00BBBB; font-weight: bold;'>-h</span>, <span style='color: #00BBBB; font-weight: bold;'>--help</span>
          Print help (see a summary with '-h')
</code></pre>

`````

### `yamark format`

`````{=html}
<pre class="yamark-cli-help"><code><span style='color: #5555FF; font-weight: bold;'>Usage:</span> <span style='color: #00BBBB; font-weight: bold;'>yamark format</span> <span style='color: #555555;'>[OPTIONS]</span> <span style='color: #555555;'>[PATHS]...</span>

<span style='color: #5555FF; font-weight: bold;'>Arguments:</span>
  <span style='color: #555555;'>[PATHS]...</span>

<span style='color: #5555FF; font-weight: bold;'>Options:</span>
      <span style='color: #00BBBB; font-weight: bold;'>--check</span>
      <span style='color: #00BBBB; font-weight: bold;'>--diff</span>
      <span style='color: #00BBBB; font-weight: bold;'>--diagnostics</span>
      <span style='color: #00BBBB; font-weight: bold;'>--stdin-file-path</span><span style='color: #555555;'> &lt;PATH&gt;</span>
      <span style='color: #00BBBB; font-weight: bold;'>--config</span><span style='color: #555555;'> &lt;PATH&gt;</span>
      <span style='color: #00BBBB; font-weight: bold;'>--wrap</span><span style='color: #555555;'> &lt;WRAP&gt;</span>                  [default: 72]
      <span style='color: #00BBBB; font-weight: bold;'>--canonical</span>
      <span style='color: #00BBBB; font-weight: bold;'>--preserve-footnotes</span>
      <span style='color: #00BBBB; font-weight: bold;'>--line-width</span><span style='color: #555555;'> &lt;LINE_WIDTH&gt;</span>      [default: 80]
      <span style='color: #00BBBB; font-weight: bold;'>--prose-width</span><span style='color: #555555;'> &lt;PROSE_WIDTH&gt;</span>    [default: 72]
      <span style='color: #00BBBB; font-weight: bold;'>--indent-width</span><span style='color: #555555;'> &lt;INDENT_WIDTH&gt;</span>  [default: 2]
      <span style='color: #00BBBB; font-weight: bold;'>--compact</span>
      <span style='color: #00BBBB; font-weight: bold;'>--skip-embedded-formatters</span>
  <span style='color: #00BBBB; font-weight: bold;'>-h</span>, <span style='color: #00BBBB; font-weight: bold;'>--help</span>                         Print help</code></pre>

`````

### `yamark render`

`````{=html}
<pre class="yamark-cli-help"><code>Render a read-only formatted view from stdin

<span style='color: #5555FF; font-weight: bold;'>Usage:</span> <span style='color: #00BBBB; font-weight: bold;'>yamark render</span> <span style='color: #555555;'>[OPTIONS]</span> <span style='color: #00BBBB; font-weight: bold;'>--stdin-file-path</span><span style='color: #555555;'> &lt;PATH&gt;</span>

<span style='color: #5555FF; font-weight: bold;'>Options:</span>
      <span style='color: #00BBBB; font-weight: bold;'>--stdin-file-path</span><span style='color: #555555;'> &lt;PATH&gt;</span>
      <span style='color: #00BBBB; font-weight: bold;'>--config</span><span style='color: #555555;'> &lt;PATH&gt;</span>
      <span style='color: #00BBBB; font-weight: bold;'>--wrap</span><span style='color: #555555;'> &lt;WRAP&gt;</span>                  [default: 72]
      <span style='color: #00BBBB; font-weight: bold;'>--canonical</span>
      <span style='color: #00BBBB; font-weight: bold;'>--preserve-footnotes</span>
      <span style='color: #00BBBB; font-weight: bold;'>--line-width</span><span style='color: #555555;'> &lt;LINE_WIDTH&gt;</span>      [default: 80]
      <span style='color: #00BBBB; font-weight: bold;'>--prose-width</span><span style='color: #555555;'> &lt;PROSE_WIDTH&gt;</span>    [default: 72]
      <span style='color: #00BBBB; font-weight: bold;'>--indent-width</span><span style='color: #555555;'> &lt;INDENT_WIDTH&gt;</span>  [default: 2]
      <span style='color: #00BBBB; font-weight: bold;'>--compact</span>
      <span style='color: #00BBBB; font-weight: bold;'>--skip-embedded-formatters</span>
  <span style='color: #00BBBB; font-weight: bold;'>-h</span>, <span style='color: #00BBBB; font-weight: bold;'>--help</span>                         Print help</code></pre>

`````

### `yamark git-filter`

`````{=html}
<pre class="yamark-cli-help"><code>Git clean/smudge filter helpers for Markdown files.

These commands read Markdown from stdin and write formatted Markdown to stdout
for Git attributes filters.

Configure the filter driver with:
  yamark git-filter adopt
  yamark git-filter join
  yamark git-filter check
  yamark git-filter setup
  yamark git-filter teardown
  git config filter.yamark-md.clean "yamark git-filter clean --stdin-filename %f"
  git config filter.yamark-md.smudge "yamark git-filter smudge --stdin-filename %f --markdown-wrap-at-column 72"

Git only runs the filter for paths matched by attributes. Put these patterns in
.git/info/attributes for personal use or .gitattributes for a shared repo:
  *.md filter=yamark-md
  *.qmd filter=yamark-md
  *.Rmd filter=yamark-md
  *.rmd filter=yamark-md

<span style='color: #5555FF; font-weight: bold;'>Usage:</span> <span style='color: #00BBBB; font-weight: bold;'>yamark git-filter</span> <span style='color: #555555;'>&lt;COMMAND&gt;</span>

<span style='color: #5555FF; font-weight: bold;'>Commands:</span>
  <span style='color: #00BBBB; font-weight: bold;'>clean</span>
  <span style='color: #00BBBB; font-weight: bold;'>smudge</span>
  <span style='color: #00BBBB; font-weight: bold;'>adopt</span>     Adopt the yamark Git filter for a shared repository
  <span style='color: #00BBBB; font-weight: bold;'>join</span>      Join a repository that has already adopted the yamark Git filter
  <span style='color: #00BBBB; font-weight: bold;'>check</span>     Check committed yamark Git filter blobs round-trip safely
  <span style='color: #00BBBB; font-weight: bold;'>setup</span>     Configure the yamark Git filter in a repository
  <span style='color: #00BBBB; font-weight: bold;'>teardown</span>  Remove the local yamark Git filter setup from a repository
  <span style='color: #00BBBB; font-weight: bold;'>help</span>      Print this message or the help of the given subcommand(s)

<span style='color: #5555FF; font-weight: bold;'>Options:</span>
  <span style='color: #00BBBB; font-weight: bold;'>-h</span>, <span style='color: #00BBBB; font-weight: bold;'>--help</span>
          Print help (see a summary with '-h')</code></pre>

`````

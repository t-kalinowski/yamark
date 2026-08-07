---
title: CLI Help
execute:
  echo: false
  warning: false
  message: false
  comment: ""
---



These captures come from the Yamark binary built from this repository. Start
with [Usage](usage.qmd) for common commands; use [Reference](reference.qmd) for
configuration, directives, and supported syntax.

## `yamark`

`````{=html}
<pre class="yamark-cli-help"><code>A formatter for YAML and Markdown.

Run `yamark &lt;COMMAND&gt; --help` for command-level help.

Usage: yamark &lt;COMMAND&gt;

Commands:
  format
  git-filter  Git clean/smudge filter helpers for Markdown files
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help (see a summary with '-h')
</code></pre>

`````

## `yamark format`

`````{=html}
<pre class="yamark-cli-help"><code>Usage: yamark format [OPTIONS] [PATHS]...

Arguments:
  [PATHS]...

Options:
      --check
      --diff
      --diagnostics
      --verify                       Reparse changed YAML and reject invalid or value-changing output
      --stdin-file-path &lt;PATH&gt;
      --config &lt;PATH&gt;
      --wrap &lt;WRAP&gt;                  [default: 72]
      --canonical
      --preserve-footnotes
      --line-width &lt;LINE_WIDTH&gt;      [default: 80]
      --prose-width &lt;PROSE_WIDTH&gt;    [default: 72]
      --indent-width &lt;INDENT_WIDTH&gt;  [default: 2]
      --compact
      --skip-embedded-formatters
  -h, --help                         Print help</code></pre>

`````

## `yamark git-filter`

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

Usage: yamark git-filter &lt;COMMAND&gt;

Commands:
  clean
  smudge
  adopt     Adopt the yamark Git filter for a shared repository
  join      Join a repository that has already adopted the yamark Git filter
  check     Check committed yamark Git filter blobs round-trip safely
  setup     Configure the yamark Git filter in a repository
  teardown  Remove the local yamark Git filter setup from a repository
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help (see a summary with '-h')</code></pre>

`````

---
title: CLI Help
execute:
  echo: false
  warning: false
  message: false
  comment: ""
---



## `yamark`

`````{=html}
<pre class="yamark-cli-help"><code>An ultra-fast YAML and Markdown formatter.

Run `yamark &lt;COMMAND&gt; --help` for command-level help.

<span style='font-weight: bold; text-decoration: underline;'>Usage:</span> <span style='font-weight: bold;'>yamark</span> &lt;COMMAND&gt;

<span style='font-weight: bold; text-decoration: underline;'>Commands:</span>
  <span style='font-weight: bold;'>format</span>
  <span style='font-weight: bold;'>git-filter</span>  Git clean/smudge filter helpers for Markdown files
  <span style='font-weight: bold;'>help</span>        Print this message or the help of the given subcommand(s)

<span style='font-weight: bold; text-decoration: underline;'>Options:</span>
  <span style='font-weight: bold;'>-h</span>, <span style='font-weight: bold;'>--help</span>
          Print help (see a summary with '-h')
</code></pre>

`````

## `yamark format`

`````{=html}
<pre class="yamark-cli-help"><code><span style='font-weight: bold; text-decoration: underline;'>Usage:</span> <span style='font-weight: bold;'>yamark format</span> [OPTIONS] [PATHS]...

<span style='font-weight: bold; text-decoration: underline;'>Arguments:</span>
  [PATHS]...

<span style='font-weight: bold; text-decoration: underline;'>Options:</span>
      <span style='font-weight: bold;'>--check</span>
      <span style='font-weight: bold;'>--diff</span>
      <span style='font-weight: bold;'>--diagnostics</span>
      <span style='font-weight: bold;'>--stdin-file-path</span> &lt;PATH&gt;
      <span style='font-weight: bold;'>--config</span> &lt;PATH&gt;
      <span style='font-weight: bold;'>--wrap</span> &lt;WRAP&gt;
      <span style='font-weight: bold;'>--canonical</span>
      <span style='font-weight: bold;'>--preserve-footnotes</span>
      <span style='font-weight: bold;'>--line-width</span> &lt;LINE_WIDTH&gt;      [default: 80]
      <span style='font-weight: bold;'>--prose-width</span> &lt;PROSE_WIDTH&gt;    [default: 72]
      <span style='font-weight: bold;'>--indent-width</span> &lt;INDENT_WIDTH&gt;  [default: 2]
      <span style='font-weight: bold;'>--compact</span>
      <span style='font-weight: bold;'>--skip-embedded-formatters</span>
  <span style='font-weight: bold;'>-h</span>, <span style='font-weight: bold;'>--help</span>                         Print help</code></pre>

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

<span style='font-weight: bold; text-decoration: underline;'>Usage:</span> <span style='font-weight: bold;'>yamark git-filter</span> &lt;COMMAND&gt;

<span style='font-weight: bold; text-decoration: underline;'>Commands:</span>
  <span style='font-weight: bold;'>clean</span>
  <span style='font-weight: bold;'>smudge</span>
  <span style='font-weight: bold;'>adopt</span>     Adopt the yamark Git filter for a shared repository
  <span style='font-weight: bold;'>join</span>      Join a repository that has already adopted the yamark Git filter
  <span style='font-weight: bold;'>check</span>     Check committed yamark Git filter blobs round-trip safely
  <span style='font-weight: bold;'>setup</span>     Configure the yamark Git filter in a repository
  <span style='font-weight: bold;'>teardown</span>  Remove the local yamark Git filter setup from a repository
  <span style='font-weight: bold;'>help</span>      Print this message or the help of the given subcommand(s)

<span style='font-weight: bold; text-decoration: underline;'>Options:</span>
  <span style='font-weight: bold;'>-h</span>, <span style='font-weight: bold;'>--help</span>
          Print help (see a summary with '-h')</code></pre>

`````

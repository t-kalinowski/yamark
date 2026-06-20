---
title: Reference
description: File types, options, directives, configuration, and safety notes.
execute:
  echo: false
  warning: false
  message: false
  comment: ""
---

## File types

Path-aware formatting supports:

- `.yaml` and `.yml`: format the whole YAML stream.
- Markdown-like extensions `.md`, `.qmd`, `.Rmd`, and `.rmd`: format Markdown
  and YAML front matter.
- Markdown fenced code blocks marked `yaml` or `yml`: format the fence contents
  as YAML.
- Markdown fenced code blocks marked `markdown` or `md`: format the fence
  contents recursively as Markdown.
- Markdown fenced code blocks marked Python, R, JSON, GraphQL, CSS, HTML,
  JavaScript, TypeScript, and related aliases: format through Ruff, Air, or
  Prettier when a matching embedded formatter is available.
- `.py`, `.R`, and `.r`: format `#|` hashpipe YAML comment blocks,
  explicit embedded Markdown comment blocks, marked Markdown string literals,
  and marked string literals through embedded formatters. Yamark does not
  format the surrounding source code.
- YAML scalar values tagged `!markdown` or marked with `# fmt: markdown`:
  format the scalar value as Markdown.
- YAML literal block scalar values marked with `# fmt: <name>` or
  `# fmt: embedded <name>`: format the scalar through a configured or built-in
  embedded formatter.

## What's supported

The syntax surface Yamark formats, preserves, or rejects.

### Markdown

| Syntax | Behavior |
| --- | --- |
| Paragraphs | Wrapped by column, paragraph, sentence, or not at all, depending on `--wrap` and document options. |
| ATX headings | Normalized spacing; heading attributes are preserved and compacted when safe. |
| Lists and task lists | List paragraphs reflow with container indentation accounted for; GFM task markers are preserved. |
| Blockquotes | Wrapped recursively when the blockquote shape is supported. |
| Footnote blocks | Footnote definitions wrap by default; `--preserve-footnotes` preserves definitions byte-for-byte. |
| Reference links | Reference-style links are atomic wrapping tokens; reference definitions are preserved without relocation. |
| Nested image links | Simple nested image labels and long image/link destinations are normalized and wrapped when safe. |
| Pandoc citations | Citation spans such as `[@key]` and `[-@key]` are treated as protected inline tokens during wrapping. |
| Quarto divs | Supported fenced div bodies are formatted recursively. |
| YAML fences | Fenced `yaml` and `yml` blocks are formatted as YAML unless locally skipped. |
| Markdown fences | Fenced `markdown` and `md` blocks are formatted recursively. |
| External source fences | Python, R, JSON, GraphQL, CSS, HTML, JavaScript, TypeScript, and related aliases can run embedded formatters when available. |
| Document Markdown options | `editor_options.markdown.{wrap,canonical,footnotes}` in front matter controls the Markdown body and recursive Markdown content that follows. |
| GFM pipe tables | Supported tables are aligned by display width; Git clean/smudge filters use compact pipe-table output. |
| Pandoc tables | Simple, grid, and multiline tables are normalized when Yamark can parse the table shape. |
| Definition lists | Supported Pandoc definition lists normalize marker spacing and wrap definitions. |
| Display math | Own-line `$$` display math blocks are preserved byte-for-byte. |
| Raw HTML, TeX, Hugo shortcodes, table captions, line blocks | Preserved when Yamark does not have a safe rewrite. |
| Template spans | Known template delimiter spans are preserved. Configure more delimiters in `yamark.toml` or with `fmt: template.delimiters`. |

### YAML

| Syntax | Behavior |
| --- | --- |
| Block mappings and sequences | Normalized indentation and spacing; keys are not reordered. |
| Flow expansion | Flow mappings and sequences normalize spacing; multiline or over-width flow collections expand to block style when safe. |
| Compact collections | `--compact`, `[format].compact`, or `fmt: compact` can collapse eligible block collections to flow style. |
| Scalar folding | Long safe prose scalars can become folded block scalars and wrap to `--prose-width`. Existing folded prose can be rewrapped. |
| Literal scalars | Preserved unless explicitly marked as Markdown or an embedded formatter target. |
| Quoted scalars with hard newlines | Can emit literal block scalars when the YAML value is unchanged. |
| Unsafe scalar quoting | Plain scalars that would change YAML meaning in block or flow context are quoted. |
| Bool/null normalization | Plain core booleans and nulls normalize to YAML 1.2 spellings such as `true`, `false`, and `null`; explicit core tags normalize when safe. |
| Tags and anchors | Custom tags, anchors, aliases, and tag/anchor order are preserved. Redundant core collection tags may be removed when syntax already implies the type. |
| Comments and directives | Comments are preserved around supported nodes. `fmt:` comments control skip, Markdown, compact, table, template, and embedded formatter behavior. |
| Markdown option directives | `fmt: canonical=true` and `fmt: wrap=sentence` can tune the next already-marked Markdown scalar without marking unrelated scalars. |
| Duplicate keys | Not validated or reordered. Yamark may still format the associated values. |
| Layout repair | A narrow unmatched `[` or `{` hint can collapse a block collection to flow style; a newline inside a flow collection can expand it. |
| BOM and line endings | UTF-8 BOM and dominant CRLF or CR line endings are preserved. UTF-16 BOM input is rejected. |
| Tab indentation | YAML with tabs in indentation is preserved when no active targeted formatting would need to interpret that region; targeted formatting rejects unsupported tab-indented YAML. |

### Source files

| Syntax | Behavior |
| --- | --- |
| `#|` hashpipe YAML comments in `.py`, `.R`, and `.r` | Consecutive own-line `#|` comments are parsed as YAML, formatted, and emitted back with the original comment prefix. |
| Embedded Markdown source comments and strings | Explicit `fmt: markdown` targets are formatted as Markdown with source-comment or string indentation accounted for. |
| External source strings | Explicit `fmt: <name>` targets format string literals through configured or built-in external formatters with string indentation accounted for. |
| Surrounding source code | Preserved. Yamark does not format Python or R source outside explicit embedded targets. |
| Quarto chunk headers | Simple chunk header options such as `echo=FALSE` can be promoted to `#| echo: false` option lines when the chunk is formatted. |
| Quarto chunk skips | A leading `#| fmt: skip` preserves that chunk locally. |

### External formatters

External formatters only run for explicit targets or known Markdown fence
languages. Default embedded formatter aliases (`ruff`, `air`, `mdformat`, and
the `prettier` family) are optional: a missing executable leaves the target
unchanged and reports a note only with `--diagnostics`. See
[`[embedded]`](#embedded) for configuration and stricter custom-formatter
behavior, and `--skip-embedded-formatters` to disable them all.

## Command modes

The default mode writes changes in place:

```sh
yamark format docs/
```

Use check, diff, or stdin mode when an integration should not mutate files:

```sh
yamark format --check docs/
yamark format --diff docs/
yamark format --stdin-file-path config.yaml < config.yaml
```

`--check` and `--diff` do not write files. Both exit `1` when any selected file
would change. `--diff` prints unified diffs to stdout. Summaries and
diagnostics go to stderr. Stdin mode writes only formatted content to stdout
and rejects additional paths.

## CLI options

Rendered `--help` captures are on the [CLI Help](cli-help.qmd) page.

```sh
yamark format --wrap none docs/
yamark format --wrap paragraph docs/
yamark format --wrap sentence docs/
yamark format --wrap 88 docs/
yamark format --canonical docs/
yamark format --preserve-footnotes docs/
yamark format --line-width 100 docs/
yamark format --prose-width 80 docs/
yamark format --indent-width 4 docs/
yamark format --compact docs/
yamark format --diagnostics docs/
yamark format --skip-embedded-formatters docs/
yamark format --config path/to/yamark.toml docs/
```

| Option | Effect |
| --- | --- |
| `--wrap none` | Disable Markdown prose wrapping. |
| `--wrap paragraph` | Put each Markdown paragraph on one line. |
| `--wrap sentence` | Put each Markdown sentence on its own line. |
| `--wrap <n>` | Wrap Markdown prose to column `<n>`. |
| `--canonical` | Enable canonical Markdown spelling for safe Markdown spans. YAML scalar spelling has its own rules, with limited canonical-mode behavior for short folded prose rewrapping. |
| `--preserve-footnotes` | Preserve Markdown footnote definitions byte-for-byte. |
| `--line-width <n>` | Set structural YAML width. Flow collections expand when they exceed it. |
| `--prose-width <n>` | Set folded YAML scalar prose width. |
| `--indent-width <n>` | Set YAML indentation width for emitted block collections. |
| `--compact` | Enable eligible YAML block-to-flow collection compaction. |
| `--diagnostics` | Print notes for supported preserved constructs and optional embedded formatter skips. |
| `--skip-embedded-formatters` | Disable external embedded formatters while keeping Yamark's own Markdown, YAML, front matter, and recursive Markdown fence formatting active. |
| `--config <path>` | Use one explicit `yamark.toml` for all selected files. |

Git filter subcommands are documented in [Git Filter](git-filter.qmd):

```sh
yamark git-filter adopt
yamark git-filter join
yamark git-filter check
yamark git-filter clean --stdin-filename docs/file.md
yamark git-filter smudge --stdin-filename docs/file.md --markdown-wrap-at-column 72
```

## Configuration

Yamark discovers `yamark.toml` from each formatted file's directory upward. Pass
`--config` to use one explicit config file for all selected paths.

Top-level tables are `[format]`, `[template]`, `[embedded]`, and `[paths]`.
Unknown keys fail fast.

### `[format]`

```toml
[format]
compact = true
markdown_horizontal_rule = "***"
```

- `compact`: boolean. Enables YAML compact mode for path-aware formatting.
  The CLI `--compact` flag enables compact mode even when config sets it to
  `false`.
- `markdown_horizontal_rule`: string. Must be `"---"` or `"***"`.

### `[template]`

Template delimiters mark regions Yamark must preserve because they can change
the host language after rendering. Default delimiters are `{{ }}`, `{% %}`,
`{# #}`, and `<% %>`.

```toml
[template]
add_delimiters = [
  { open = "<<", close = ">>" }
]
```

Use `replace_delimiters` to replace the defaults for the matching scope:

```toml
[template]
replace_delimiters = [
  { open = "[[", close = "]]" }
]
```

Each delimiter entry must contain non-empty `open` and `close` strings.

### `[embedded]`

Embedded formatter entries map directive names to external stdin/stdout
formatters.

```toml
[embedded.python]
formatter = "ruff"

[embedded.r]
formatter = "air"

[embedded.sql]
formatter = { command = ["sqlfmt", "--filename", "{path}"], path_suffix = ".sql" }
```

Built-in shorthands include `ruff`, `air`, `mdformat`,
`prettier-json`, `prettier-jsonc`, `prettier-json5`, `prettier-graphql`,
`prettier-css`, `prettier-scss`, `prettier-less`, `prettier-postcss`,
`prettier-html`, `prettier-js`, `prettier-jsx`, `prettier-ts`, and
`prettier-tsx`.

Custom formatter commands are argv arrays, not shell strings. `{path}` must be
a complete argv item. `path_suffix` is appended to the synthetic path Yamark
passes to the formatter.

Missing executable behavior depends on how the formatter is registered.
Default Markdown fence formatters, including Ruff, Air, and Prettier aliases,
are optional: if the executable is missing, Yamark leaves the target unchanged
and prints `missing optional embedded formatter ...; preserved source` only
with `--diagnostics`. Configured formatters are stricter unless their command
starts with `ruff`, `air`, `mdformat`, or `prettier`; a missing executable is an
error and leaves the file unchanged.

### `[paths]`

Path keys are relative to the config file directory and must not contain `..`.
Matching path-specific config can add or replace template delimiters.

```toml
[paths."docs".template]
add_delimiters = [
  { open = "<<", close = ">>" }
]

[paths."prompts".embedded_markdown.template]
add_delimiters = [
  { open = "[[", close = "]]" }
]
```

`paths.<path>.template` affects generic template detection for matching files.
`paths.<path>.embedded_markdown.template` affects Markdown embedded inside
source strings or comments.

## Document Markdown options

Markdown front matter can set document-local Markdown options under
`editor_options.markdown`:

```markdown
---
editor_options:
  markdown:
    wrap: sentence
    canonical: true
    footnotes: preserve
---
This is __strong__. This is _emphasis_.
```

The supported keys are:

| Key | Values |
| --- | --- |
| `wrap` | `none`, `paragraph`, `sentence`, or a positive integer column. |
| `canonical` | `true`, `false`, `yes`, `no`, `1`, or `0`. |
| `footnotes` | `wrap`, `format`, `preserve`, `none`, `true`, `false`, `yes`, `no`, `1`, or `0`. |

These options apply after the front matter: to the Markdown body, recursive
`markdown` or `md` fences, and Markdown-valued YAML scalars in nested YAML
regions. `--preserve-footnotes` still wins over front matter and preserves
footnote definitions byte-for-byte.

Yamark also reads `editor.markdown` for compatibility, but only when
`editor_options.markdown` is absent. The two tables are not merged. Git filter
subcommands use their own Markdown wrapping options and do not read these
document options.

## Directives

Directives are comments whose trimmed content starts with `fmt:`.

- Markdown body directives use own-line HTML comments:
  `<!-- fmt: wrap=sentence scope=file -->`.
- YAML and source-file directives use own-line hash comments:
  `# fmt: markdown`.
- YAML also accepts same-line `# fmt: table` and `# fmt: compact table` on a
  collection parent.

Scopes:

| Scope | Meaning |
| --- | --- |
| `scope=next` | Apply to the next supported target. This is the default for YAML scalar, compact, table, template, and embedded formatter target directives. |
| `scope=from-here` | Apply from the directive until another directive changes the state. |
| `scope=file` | Apply to the whole file, including supported nodes before the directive. |

Directive grammar details:

| Form | Effect |
| --- | --- |
| `fmt: compact` | Enable compact YAML collection output for the next target. |
| `fmt: compact false` or `fmt: compact=false` | Disable compact YAML collection output for the next target. |
| `fmt: compact scope=from-here` | Enable compact output until another compact directive changes the state. |
| `fmt: compact=false scope=file` | Disable compact output for the whole YAML file or source-file hashpipe YAML stream. |
| `fmt: markdown wrap=sentence` | Mark the next YAML scalar as Markdown and set its Markdown options. |
| `fmt: canonical=true` or `fmt: wrap=sentence` | Set Markdown options for the next already-marked Markdown scalar without marking unrelated scalars. |
| `fmt: skip` | Preserve the next supported target. |
| `fmt: off` and `fmt: on` | Preserve a region, then resume formatting. |

Skip a whole file:

```yaml
# fmt: skip file
z:     [  1,2,3]
```

```markdown
<!-- fmt: skip file -->
#   Keep this heading ##
```

Skip the next YAML node:

```yaml
normal: [1, 2, 3]
# fmt: skip
manual:
    -   [ 1,2,3]
    -   [ 4,5,6]
```

Disable formatting for a region:

```yaml
# fmt: off
manual: [[ 1,2,3], [4,5,6]]
# fmt: on
```

The same skip and region controls work in Markdown and source comments:

```markdown
normal paragraph

<!-- fmt: skip -->
#   Manual heading ##

<!-- fmt: off -->
*   keep
    this
    list
<!-- fmt: on -->
```

```python
# fmt: skip
# #   Manual Markdown ##

# fmt: off
# *   keep
#     this
#     list
# fmt: on
```

Set Markdown options:

```markdown
<!-- fmt: canonical -->
<!-- fmt: wrap=72 canonical=true scope=file -->
<!-- fmt: scope=from-here wrap=none footnotes=preserve -->
```

```yaml
# fmt: markdown wrap=sentence canonical=true
body: "_First sentence_. Second sentence?"
```

Option-only YAML directives tune the next already-marked Markdown scalar
without making the next plain scalar a Markdown target:

```yaml
# fmt: canonical=true
first: !markdown "This is __strong__."
second: "This is __not__ Markdown."
```

`wrap` accepts `none`, `paragraph`, `sentence`, or a positive integer.
`canonical` accepts `true`, `false`, `yes`, `no`, `1`, or `0`; the bare
`canonical` token means `canonical=true`. `footnotes` accepts `wrap`, `format`,
`preserve`, `none`, `true`, `false`, `yes`, `no`, `1`, or `0`.

Mark a scalar as Markdown:

```yaml
# fmt: markdown
body: "A paragraph with [a link](https://example.com)."

tagged: !markdown "This value is also Markdown."
```

Register additional template delimiters:

```yaml
# fmt: template.delimiters "<<" ">>"
body: "<< keep this template span >>"
```

Enable compact YAML collection output:

```yaml
# fmt: compact
tags:
  - llm
  - authoring
```

Align a following sequence of flow mappings as a table:

```yaml
# fmt: table
- {name: a,         type: int,    default: 0}
- {name: long_name, type: string, default: ""}
```

Collapse eligible block mapping rows before table alignment:

```yaml
rows: # fmt: compact table
  - name: a
    kind: scalar
  - name: longer
    kind: sequence
```

Format source-file `#|` hashpipe YAML comment blocks:

```r
#| name: demo
#| launcher:
#|  vanilla: true
#|  default-packages: [base,utils]

main <- function() NULL
```

becomes:

```r
#| name: demo
#| launcher:
#|   vanilla: true
#|   default-packages: [base, utils]

main <- function() NULL
```

Format a YAML literal scalar with an embedded formatter:

```yaml
# fmt: embedded python
script: |
  def f(x):return x+1

# fmt: r
analysis: |
  f <- function(x)x+1
```

Format embedded Markdown in source strings or comments:

```python
# fmt: markdown wrap=sentence
PROMPT = """
#   Title

Read the diff and report correctness issues. Prefer specific examples.
"""
```

Format an embedded R string in Python:

```python
# fmt: r
source = """
f <- function(x)x+1
"""
```

Markdown fenced chunks can opt out locally:

````markdown
```yaml fmt: skip
items: [a,b]
```

```r
#| fmt: skip
f <- function(x)x+1
```
````

Quarto chunk header options can be promoted to `#|` option lines when an
embedded formatter handles the chunk:

The opening chunk line `` ```{r, echo=FALSE, fig.width=8}`` becomes an
unadorned `` ```{r}`` header plus option lines:

```text
#| echo: false
#| fig.width: 8
```

Use `#| fmt: skip` at the top of a Quarto chunk to preserve that chunk locally.

## Layout repair

Yamark accepts two narrow forms of ill-formed but obvious YAML as layout hints.
They let you toggle a collection between block and flow style by typing one
character or inserting one newline.

### Collapse to flow with `[` or `{`

Drop an unmatched `[` or `{` where a block sequence or mapping value starts.
Yamark removes the opener and emits the collection in flow style when the
result fits the active line width.

Before:

```yaml
tags: [
  - llm
  - authoring
  - formats
```

After:

```yaml
tags: [llm, authoring, formats]
```

The opener can also sit on the line after an empty mapping value header:

```yaml
tags:
[
  - llm
  - authoring
  - formats
```

The same hint works on mappings:

```yaml
config: {
  host: example.com
  port: 8080
  tls: true
```

becomes:

```yaml
config: {host: example.com, port: 8080, tls: true}
```

### Expand to block with a newline

A line break inside a flow collection is taken as a request to expand it to
block style.

Before:

```yaml
- {a: b,
   c: d}
```

After:

```yaml
- a: b
  c: d
```

### Rejection rules

A repair is accepted only when the hint targets one independent collection, the
collapsed output fits the configured `--line-width`, and the reparse and
semantic-equivalence check passes. If not, Yamark reports the original parse
error and leaves the file unchanged.

## Safety

Yamark is a formatter, not a validator. It formats supported regions and keeps
unsupported regions unchanged when a safe rewrite is not available.

For YAML regions it changes, Yamark parses the input, formats it, and reparses
the output before writing. For Markdown and Markdown-valued YAML scalars, the
replacement is written only when the original and replacement parse
equivalently. For configured embedded formatters, only explicitly marked
targets can change.

If parsing, formatting, a required equivalence check, UTF-8 decoding, external
formatter execution, or I/O fails, the affected file is left unchanged and a
diagnostic is reported.

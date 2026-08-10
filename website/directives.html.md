---
# fmt: skip file
title: Directives
description: Tell Yamark what to format, skip, preserve, or configure from inside the file.
---

<!-- fmt: skip file -->

The source file is the primary interface. Put a small `fmt:` directive next to
the text it controls, then let an editor or repository integration run Yamark.
Because directives are comments, the formatting choice stays with the text.
The comment marker depends on the file:

| File | Directive comment |
| --- | --- |
| Markdown and Quarto | `<!-- fmt: ... -->` |
| YAML, Python, and R | `# fmt: ...` |

Quarto chunk options use `#|`; for example, `#| fmt: skip`.

## Target the next value or block

Most directives apply to the next supported target. In a YAML file, that target
is usually a scalar node. In a Python or R file, it is usually a string literal
or a contiguous source-comment block. The directive does not format the
surrounding Python or R source code.

## YAML files

Mark a scalar as Markdown and choose sentence wrapping for that scalar:

```yaml
# fmt: markdown wrap=sentence
description: "Explain the project in a few clear sentences."
other_value: "This is ordinary YAML, not Markdown."
```

The directive attaches to `description`, the next scalar. A tagged scalar is
another way to mark Markdown when the setting should travel with the value:

```yaml
instructions: !markdown |
  # Review

  Read the diff and report concrete correctness issues.
```

Use the same placement for an embedded formatter. The formatter name is the
language after `fmt:`:

```yaml
# fmt: python
script: |
  def check(items):return all(item > 0 for item in items)
```

YAML also supports directives on a collection parent when the setting describes
the collection rather than a scalar:

```yaml
# fmt: table
- {name: alpha, type: string, default: ""}
- {name: beta, type: integer, default: 0}
```

## Python files

Put `# fmt: markdown` immediately before the string literal or comment block
that contains Markdown:

```python
# fmt: markdown wrap=sentence
REVIEW_PROMPT = """
# Review

Read the diff and report concrete correctness issues. Prefer specific examples.
"""
```

The `#` lines inside the string are Markdown content. Python outside the
marked target is left for a Python formatter such as Ruff.

The same directive can target a Markdown comment block:

```python
# fmt: markdown
# # Filters
#
# Apply the most specific filter first.
#
# - Each filter is a single expression.
```

For a non-Markdown embedded language, use its formatter name:

```python
# fmt: r
analysis = """
f <- function(x) x + 1
"""
```

## R files

R uses the same `# fmt:` marker. It can target a raw string literal used for a
vignette, help text, or a Shiny message:

```r
# fmt: markdown wrap=sentence
help_text <- r"(
# Filters

Apply the most specific filter first. Explain why a filter matched.
)"
```

It can also target a contiguous Markdown comment block:

```r
# fmt: markdown
# # Filters
#
# Apply the most specific filter first.
#
# - Each filter is a single expression.
```

For source code inside a YAML literal scalar, use the language formatter name
in the same way as in a YAML file:

```yaml
# fmt: r
analysis: |
  f <- function(x)x+1
```

## Control a region or a whole file

The scope is part of the directive. `next` is the default for target directives.
Use `from-here` for the rest of a region, and `file` for a document-wide
setting:

| Scope | Applies to |
| --- | --- |
| `scope=next` | The next supported string, scalar, collection, or comment block. |
| `scope=from-here` | Supported content from this point until another directive changes the setting. |
| `scope=file` | The whole file, including supported content before the directive. |

Skip one target:

```yaml
normal: [1, 2, 3]
# fmt: skip
manual:
    -   [ 1,2,3]
```

Preserve a region and resume formatting afterward:

```markdown
<!-- fmt: off -->
#   Keep this heading ##

<!-- fmt: on -->
```

Skip a whole file by putting the file directive at the top. YAML, Python, and R
use the same hash-comment form:

```text
# fmt: skip file
```

Markdown uses an HTML comment:

```markdown
<!-- fmt: skip file -->
```

A whole-file skip is useful for generated files or files whose layout is
intentionally outside Yamark's scope.

## Set file-specific Markdown options

Use `scope=file` when the file should have a different Markdown policy from the
repository default:

```markdown
<!-- fmt: wrap=sentence:72 scope=file -->
```

This puts one sentence per line and wraps long sentences at column 72. Other
file-wide Markdown options include `canonical=true` and
`footnotes=preserve`:

```markdown
<!-- fmt: canonical=true footnotes=preserve scope=file -->
```

For a YAML file or a hashpipe YAML stream in Python or R, use the same scope to
set YAML behavior for the whole file:

```yaml
# fmt: compact=false scope=file
```

An option-only directive such as `# fmt: canonical=true` adjusts the next
already-marked Markdown scalar. It does not turn an ordinary YAML scalar into a
Markdown target:

```yaml
# fmt: canonical=true
first: !markdown "This is __strong__ Markdown."
second: "This is ordinary YAML text."
```

## Arrange for Yamark to run automatically

Once directives are in the files, configure Yamark to run as part of the
workflow:

- In an editor, enable format on save. The [Editors](editors.qmd) page shows
  the VS Code and Positron settings. This is the most direct way to see a
  directive take effect while editing.
- In a pre-commit hook, run `yamark format`. In CI, use
  `yamark format --check` to verify files without changing them.

The experimental [Git Filter](git-filter.qmd) serves a narrower workflow: it
stores selected Markdown files sentence-per-line while keeping the working tree
column-wrapped.

## When a repository default is better

Use `yamark.toml` for settings that should apply broadly rather than to one
piece of text. Keep local exceptions in directives so they stay beside the
content they explain.

For example, a repository can choose sentence-per-line Markdown and compact
YAML collections by default:

```toml
[format]
wrap = "sentence:88"
compact = true
```

Yamark discovers `yamark.toml` from each formatted file's directory upward.
Use the [Configuration reference](reference-config.qmd) for the full
`[format]`, `[template]`, `[embedded]`, and `[paths]` schema.

## For one-off runs

Command-line options are useful for trying a policy, checking files in CI, or
overriding a repository default for one invocation:

```sh
yamark format --wrap sentence:72 docs/
yamark format --compact config.yaml
yamark format --check docs/
```

Use the [Directive syntax reference](reference-directives.qmd) for every
accepted form, scope, and value.

Use the [Command-line reference](cli-help.qmd) for the accepted arguments and
[Usage](usage.qmd) for installation and common commands.

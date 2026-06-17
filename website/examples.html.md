---
# fmt: skip file
title: Showcase
description: >-
  Real before/after examples covering the file types and directives Yamark
  exists for.
---

<!-- fmt: skip file -->



Yamark formats the files that sit between code and docs: skill files, prompt
templates, build matrices, configs, and any source file that hosts a prose blob.
Each section below demonstrates one capability with the smallest before/after
that shows it off.

## Markdown with YAML frontmatter

The shape of an agent skill, a prompt template, or a Cursor rule: typed
frontmatter on top, free-form Markdown body underneath. Yamark formats the
frontmatter as YAML and the body as Markdown in one pass.

:::: {.showcase-before-after}
**Before**

```markdown
---
name: review-pr
description: Review a pull request and recommend changes inline with the project's style guide, focusing on correctness, readability, and tests.
tags: [review, pull-request, code]
---

# Review

Read the diff and flag anything that violates the style guide, has obvious correctness issues, or looks untested. Prefer specific suggestions over vague concerns.
```

**After**

```markdown
---
name: review-pr
description: >-
  Review a pull request and recommend changes inline with the project's
  style guide, focusing on correctness, readability, and tests.
tags: [review, pull-request, code]
---

# Review

Read the diff and flag anything that violates the style guide, has
obvious correctness issues, or looks untested. Prefer specific
suggestions over vague concerns.
```
::::

## Markdown-valued YAML scalars

A `prompts.yaml` or `agents.yaml` where each entry's value is a
Markdown blob. Tag the value `!markdown` (or add `# fmt: markdown`) and
Yamark wraps the value as Markdown - picking folded or literal block
style based on the content.

:::: {.showcase-before-after}
**Before**

```yaml
agents:
  reviewer:
    instructions: !markdown "Focus on correctness and tests. Flag any change that lacks a regression test. Prefer concrete suggestions: name the function, name the case."
  summarizer:
    instructions: !markdown |
      You write release notes.

      - One bullet per user-visible change.
      - No internal refactors.
```

**After**

```yaml
agents:
  reviewer:
    instructions: !markdown |
      Focus on correctness and tests. Flag any change that lacks a regression
      test. Prefer concrete suggestions: name the function, name the case.
  summarizer:
    instructions: !markdown |
      You write release notes.

      - One bullet per user-visible change.
      - No internal refactors.
```
::::

The prose-only value is rewrapped as Markdown. The list keeps a literal
block (`|`) because folding would destroy the line breaks Markdown depends on.

## YAML scalar presentation

Yamark changes scalar spelling only when the parsed YAML value and tag
stay equivalent. It can simplify safe quoted strings, keep strings that
would be misread as booleans, convert hard newlines to literal blocks,
and fold long prose strings.

:::: {.showcase-before-after}
**Before**

```yaml
title: "hello"
boolish: "true"
body: "alpha\nbeta\n"
summary: "This package formats YAML front matter and configuration files while preserving semantics, comments, and repository-friendly diffs."
```

**After**

```yaml
title: hello
boolish: "true"
body: |
  alpha
  beta
summary: >-
  This package formats YAML front matter and configuration files while
  preserving semantics, comments, and repository-friendly diffs.
```
::::

Yamark doesn't validate duplicate mapping keys. It keeps pairs in source order
and may still format their values:

:::: {.showcase-before-after}
**Before**

```yaml
a: [1,2]
a: [3,4]
```

**After**

```yaml
a: [1, 2]
a: [3, 4]
```
::::

## Embedded Markdown in Python

When the prompt lives next to the code that uses it, Yamark formats
the prose without touching the surrounding Python. Mark the string with
`# fmt: markdown`:

:::: {.showcase-before-after}
**Before**

```python
# fmt: markdown
REVIEW_PROMPT = """
# Review

Read the diff and flag anything that violates the style guide, has obvious correctness issues, or looks untested.

-   Prefer specific suggestions over vague concerns.
-   Name the function and the case.
"""
```

**After**

```python
# fmt: markdown
REVIEW_PROMPT = """
# Review

Read the diff and flag anything that violates the style guide, has
obvious correctness issues, or looks untested.

- Prefer specific suggestions over vague concerns.
- Name the function and the case.
"""
```
::::

Python source outside the marked string is left untouched - run `ruff format`
for that.

## Embedded Markdown in source comments

The same directive can target a contiguous source comment block. This
is useful for generated help text or prompt snippets where a string
literal is not the right host.

:::: {.showcase-before-after}
**Before**

```python
# fmt: markdown
# #   Filters
#
# Apply one or more filters. Put the most specific filter first.
#
# -   Each filter is a single expression.
```

**After**

```python
# fmt: markdown
# # Filters
#
# Apply one or more filters. Put the most specific filter first.
#
# - Each filter is a single expression.
```
::::

## Embedded Markdown in R

The same pattern works for R raw strings. Useful for package vignettes,
package-bundled prompts, and Shiny help text that you want to keep next
to the code that renders it.

:::: {.showcase-before-after}
**Before**

```r
# fmt: markdown
help_text <- r"(
# Filters

Apply one or more filters. Filters are evaluated top-to-bottom; the first match wins, so put the most specific filter first.

-   Each filter is a single expression.
-   Multiple filters combine with AND.
)"
```

**After**

```r
# fmt: markdown
help_text <- r"(
# Filters

Apply one or more filters. Filters are evaluated top-to-bottom; the
first match wins, so put the most specific filter first.

- Each filter is a single expression.
- Multiple filters combine with AND.
)"
```
::::

## Aligned flow-mapping tables

Build matrices, parameter grids, and any sequence of homogeneous flow
mappings read better as a table. Mark the sequence with `# fmt: table`
and Yamark aligns columns by key:

:::: {.showcase-before-after}
**Before**

```yaml
# fmt: table
- {name: alpha,type: string,default: [one,two],description: "Short"}
- {name: beta,type: int,default: 0,description: "Number"}
```

**After**

```yaml
# fmt: table
- {name: alpha, type: string, default: [one, two], description: Short}
- {name: beta,  type: int,    default: 0,          description: Number}
```
::::

## Compact collections

Short, simple block collections can read better as a single flow line.
Enable `compact = true` in `yamark.toml` (or pass `--compact`) and
Yamark collapses eligible block mappings and sequences that fit the
structural width:

:::: {.showcase-before-after}
**Before**

```yaml
tags:
  - llm
  - authoring
  - formats
package:
  name: yamark
  language: rust
```

**After**

```yaml
tags: [llm, authoring, formats]
package: {name: yamark, language: rust}
```
::::

Collections with comments, aliases, tags, anchors, multiline scalars, or
block scalars stay in block style.

## Collapse to flow by typing a bracket

Drop an unmatched `[` or `{` where a block sequence or mapping value starts and
Yamark reads it as a layout hint: collapse this collection onto one line. The
file doesn't parse as you typed it, but the intent is obvious - Yamark removes
the opener and emits flow style.

:::: {.showcase-before-after}
**Before**

```yaml
tags: [
  - llm
  - authoring
  - formats
```

**After**

```yaml
tags: [llm, authoring, formats]
```
::::

The opener may also be detached onto the next line after an empty mapping value
header:

:::: {.showcase-before-after}
**Before**

```yaml
tags:
[
  - llm
  - authoring
  - formats
```

**After**

```yaml
tags: [llm, authoring, formats]
```
::::

Adjacent forms such as `tags:[` and `tags:{` are not repaired as layout hints.

The trick also works in the other direction: a newline inside an existing flow
collection is read as a request to expand it to block style.

:::: {.showcase-before-after}
**Before**

```yaml
- {a: b,
   c: d}
```

**After**

```yaml
- a: b
  c: d
```
::::

See [Reference -> Layout repair](reference.qmd#layout-repair) for the
acceptance rules.

## Recursive Markdown code fences

YAML fences are formatted as YAML. Markdown fences are formatted
recursively, so nested YAML inside nested Markdown is formatted too.

:::: {.showcase-before-after}
**Before**

`````markdown
```yaml
# fmt: table
- {name: a,type: int,default: 0}
- {name: long_name,type: string,default: ""}
```

````markdown
#   Inner

```yaml
items: [a,b]
```
````
`````

**After**

`````markdown
```yaml
# fmt: table
- {name: a,         type: int,    default: 0}
- {name: long_name, type: string, default: ""}
```

````markdown
# Inner

```yaml
items: [a, b]
```
````
`````
::::

## Prettier-backed web and data fences

When `prettier` is on `PATH`, JSON, JSONC, JSON5, GraphQL, CSS, SCSS, Less,
PostCSS, HTML, JavaScript, JSX, TypeScript, and TSX fenced blocks are handed to
it. Python and R fences go to Ruff and Air the same way:

:::: {.showcase-before-after}
**Before**

````markdown
```json
{"name":"yamark","tags":["llm","authoring","formats"],"engines":{"node":">=18"}}
```

```ts
function format(input:string):string{return input.trim()}
```
````

**After**

````markdown
```json
{
  "name": "yamark",
  "tags": ["llm", "authoring", "formats"],
  "engines": { "node": ">=18" }
}
```

```ts
function format(input: string): string {
  return input.trim();
}
```
````
::::

A missing `prettier` leaves the fence unchanged. Override or disable a
language with `[embedded.<language>]` in `yamark.toml`, or skip them all with
`--skip-embedded-formatters`.

## Embedded source in YAML literal blocks

Configure an embedded formatter for a language in `yamark.toml`
(`[embedded.r]`, `[embedded.python]`, custom), then mark a literal
block scalar with `# fmt: r` or `# fmt: python` to format the embedded
code through the external formatter:

:::: {.showcase-before-after}
**Before**

```yaml
# fmt: python
preflight: |
  def check(items):return all(x>0 for x in items)
```

**After**

```yaml
# fmt: python
preflight: |
  def check(items):
    return all(x > 0 for x in items)
```
::::

Yamark hands the scalar to the configured formatter and only writes if the
surrounding YAML round-trips cleanly.

## Markdown links, footnotes, and tables

Long inline links are split at Markdown syntax boundaries. Footnote
definitions wrap under the marker unless the document or CLI asks to
preserve them.

:::: {.showcase-before-after}
**Before**

```markdown
Please be mindful of our [code of conduct](https://github.com/quarto-dev/quarto-cli/blob/main/.github/CODE_OF_CONDUCT.md) as you interact with other community members.

Body text with a footnote.[^long]

[^long]: This footnote explains that comments were removed because people used comments to hold parsing directives and enough extra words to wrap.
```

**After**

```markdown
Please be mindful of our [code of conduct](
  https://github.com/quarto-dev/quarto-cli/blob/main/.github/CODE_OF_CONDUCT.md
) as you interact with other community members.

Body text with a footnote.[^long]

[^long]: This footnote explains that comments were removed because
  people used comments to hold parsing directives and enough extra words
  to wrap.
```
::::

Simple GFM pipe tables are aligned by display width. Git clean/smudge filters
always use compact pipe-table output.

:::: {.showcase-before-after}
**Before**

```markdown
| package | label |
|---|---|
| dplyr | tidy |
| tidyr | pivoting |
```

**After**

```markdown
| package | label    |
| ------- | -------- |
| dplyr   | tidy     |
| tidyr   | pivoting |
```
::::

---

See [Reference](reference.qmd) for the full directive list, options,
and safety model.

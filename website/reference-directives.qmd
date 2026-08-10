---
# fmt: skip file
title: Directive syntax
description: Exact fmt comment forms, targets, scopes, values, and effects.
---

<!-- fmt: skip file -->

Directives are comments whose trimmed content starts with `fmt:`. This page is
an exact syntax lookup. For a guided path through YAML, Python, and R examples,
see [Directives](directives.qmd).

## Comment forms

| Host | Form | Placement |
| --- | --- | --- |
| Markdown | `<!-- fmt: ... -->` | An own-line HTML comment. |
| YAML | `# fmt: ...` | An own-line hash comment. `fmt: table` and `fmt: compact table` may also be same-line comments on an empty-valued collection parent; a populated parent is an error. |
| Python and R | `# fmt: ...` | An own-line hash comment before a supported comment or string target. |

## Scopes

| Scope | Meaning |
| --- | --- |
| `scope=next` | Apply to the next supported target. This is the default for target-selecting, compact, table, and embedded-formatter directives. |
| `scope=from-here` | Apply from the directive until another directive changes the same state. |
| `scope=file` | Apply throughout the current parsed document or nested region, including supported nodes before the directive. At the top level, that region is the physical file. |

Not every directive accepts every scope. The tables below list the supported
forms. An unsupported or misspelled explicit scope is an error. Template
delimiter directives use the contextual rule described below instead of one
fixed default.

Scopes stop at the current parsed document or nested-region boundary. A
directive inside a recursive fence affects that fence, not its Markdown host or
sibling fences. A `scope=file` directive inside one hashpipe YAML block does not
span later blocks. `fmt: skip file` in Markdown front matter is the exception:
it preserves the outer Markdown document.

## Preserve content

| Directive | Scope | Effect |
| --- | --- | --- |
| `fmt: skip` | Next | Preserve the next supported target. `fmt: skip scope=next` is equivalent. |
| `fmt: skip file` | File | Preserve the current parsed document or nested region. `fmt: skip scope=file` is equivalent. |
| `fmt: off` | From here | Preserve content until `fmt: on`, or through the end of Markdown and source regions. `fmt: off scope=from-here` is equivalent. |
| `fmt: on` | - | Resume formatting after `fmt: off`. It does not accept an explicit scope. |

The same control uses the host's comment form:

```yaml
# fmt: skip
manual: [[ 1,2,3], [4,5,6]]
```

```markdown
<!-- fmt: off -->
*   keep
    this
    list
<!-- fmt: on -->
```

Preserve-control lifecycle checks are host-specific. YAML rejects a targetless
`skip`, a stray `on`, a nested `off`, and an `off` without a later `on`.
Markdown and source regions allow `skip` at the end and let `off` preserve
through the end of the region.

## Markdown targets and settings

| Form | Default scope | Effect |
| --- | --- | --- |
| `fmt: markdown` | Next | Mark the next supported target as Markdown: a Markdown block, including a supported fence; a YAML scalar; or a source string or comment block. |
| `fmt: markdown wrap=sentence canonical=true` | Next | Mark the next supported target as Markdown and set its options. |
| `fmt: wrap=sentence` | `scope=next` in YAML; `scope=file` in Markdown or source regions | Change wrapping for an already recognized Markdown region without marking unrelated content. |
| `fmt: canonical` | `scope=next` in YAML; `scope=file` in Markdown or source regions | Enable canonical Markdown spelling. `canonical=true` is equivalent. |
| `fmt: footnotes=preserve` | `scope=next` in YAML; `scope=file` in Markdown or source regions | Preserve footnote definitions in the affected Markdown. |

Option-only YAML directives tune the next scalar only if it is already marked
as Markdown:

```yaml
# fmt: canonical=true
first: !markdown "This is __strong__."
second: "This is __not__ Markdown."
```

For `scope=from-here` or `scope=file`, `fmt: markdown` must include at least one
actual Markdown option. A broad-scope directive changes Markdown settings; it
does not turn every plain scalar into Markdown.

Accepted option values:

| Option | Values |
| --- | --- |
| `wrap` | `none`, `paragraph`, `sentence`, `sentence:<n>`, or a positive integer. |
| `canonical` | `true`, `false`, `yes`, `no`, `1`, or `0`. A bare `canonical` means `true`. |
| `footnotes` | `wrap`, `format`, `preserve`, `none`, `true`, `false`, `yes`, `no`, `1`, or `0`. |

For `footnotes`, `wrap`, `format`, `true`, `yes`, and `1` request formatting;
`preserve`, `none`, `false`, `no`, and `0` request preservation.

## YAML collections

| Directive | Scope | Effect |
| --- | --- | --- |
| `fmt: compact` | Next | Enable block-to-flow compaction for the next eligible collection. |
| `fmt: compact false` | Next | Disable compaction for the next eligible collection. `fmt: compact=false` is equivalent. |
| `fmt: compact scope=from-here` | From here | Enable compaction until another compact directive changes the state. |
| `fmt: compact=false scope=file` | File | Disable compaction throughout the current YAML document or one source-file hashpipe YAML block. |
| `fmt: table` | Next | Align the next compatible sequence of flow mappings. |
| `fmt: compact table` | Next | Collapse eligible block mapping rows before aligning them. `fmt: table compact` and `fmt: table compact=true` are equivalent. |

`fmt: table` accepts only its default next-target scope. Compact directives
accept `next`, `from-here`, and `file`.

`compact`, `compact=true`, `compact=yes`, `compact=1`, and `compact true`
enable compaction. `compact=false`, `compact=no`, `compact=0`, and
`compact false` disable it. The separated form accepts only `true` or `false`.

## Templates and embedded formatters

| Directive | Scope | Effect |
| --- | --- | --- |
| `fmt: template.delimiters "<<" ">>"` | Inferred | Add one non-empty double-quoted delimiter pair using the placement rule below. |
| `fmt: template.delimiters "<<" ">>" scope=next` | Next | Add the pair to the next supported target. |
| `fmt: template.delimiters "<<" ">>" scope=from-here` | From here | Add the pair to following supported targets. |
| `fmt: template.delimiters "<<" ">>" scope=file` | File | Add the pair throughout the current document or nested region. |
| `fmt: embedded python` | Next | Format the next supported string or YAML literal scalar with the named formatter. |
| `fmt: python` | Next | Shorthand for `fmt: embedded python`. |

A bare `fmt: template.delimiters` directive uses the next target when placed
directly before one. When blank lines or range boundaries isolate it from
content on both sides, it applies from there onward. The directive requires an
explicit scope in every other placement.

Delimiter arguments must be non-empty double-quoted tokens; single quotes are
not accepted. Inside a token, `\"`, `\\`, `\n`, `\r`, and `\t` represent a
double quote, backslash, newline, carriage return, and tab.

Embedded formatter directives accept only `scope=next`. The name can select a
built-in alias or an entry from [`[embedded]`](reference-config.qmd#embedded).

YAML can also mark a scalar as Markdown with the `!markdown` or `!md` tag:

```yaml
body: !markdown "A paragraph with [a link](https://example.com)."
```

## Source files and fenced chunks

| Host | Directive | Effect |
| --- | --- | --- |
| Python or R comment or string | `fmt: markdown` | Format the next supported comment block or string as Markdown. Surrounding source code stays unchanged. |
| Python or R string | `fmt: <name>` or `fmt: embedded <name>` | Format the next supported string with the named embedded formatter. Surrounding source code stays unchanged. |
| Markdown fence | An opening attribute such as `` ```yaml fmt: skip `` | Preserve that fence locally. |
| Quarto source fence | `#| fmt: skip` anywhere in the initial consecutive `#|` option block | Preserve that chunk locally. |

[Supported files and syntax](reference-files.qmd#python-and-r-source-files)
describes automatic hashpipe YAML recognition. The [Markdown behavior
table](reference-files.qmd#markdown) describes Quarto fence-header promotion.

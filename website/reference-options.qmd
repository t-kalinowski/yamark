---
title: Formatting settings
description: Formatting controls, defaults, accepted values, scopes, and interactions.
---

Start here when you know which output should change but not where Yamark exposes
the control. This page groups settings by behavior; the linked
[configuration](reference-config.qmd),
[directive](reference-directives.qmd), and [command-line](cli-help.qmd) pages
give the exact syntax for each surface.

## Markdown settings

| Behavior | [Directive](reference-directives.qmd#markdown-targets-and-settings) | [Front matter](#document-markdown-options) | [`yamark.toml`](reference-config.qmd#format) | [Command line](cli-help.qmd#yamark-format) | Default |
| --- | --- | --- | --- | --- | --- |
| Prose wrapping | `wrap=` | `editor_options.markdown.wrap` | `[format].wrap` | `--wrap` | Column 72 |
| Canonical spelling | `canonical=` or `canonical` | `editor_options.markdown.canonical` | - | `--canonical` | Off |
| Footnote definitions | `footnotes=` | `editor_options.markdown.footnotes` | - | `--preserve-footnotes` | Format |
| Horizontal-rule marker | - | - | `[format].markdown_horizontal_rule` | - | `---` |

### Wrapping values

| Value | Effect |
| --- | --- |
| `none` | Do not wrap Markdown prose. |
| `paragraph` | Put each paragraph on one physical line. |
| `sentence` | Put each sentence on its own line without column wrapping. |
| `sentence:<n>` | Put each sentence on its own line and wrap long sentences to column `<n>`. |
| A positive integer | Wrap prose to that column without forcing sentence boundaries. |

`sentence:<n>` is a Yamark-specific extension. RStudio's visual writer does not
recognize the combined value.

### Canonical and footnote values

`canonical` accepts `true`, `false`, `yes`, `no`, `1`, or `0` in front matter
and directives. A bare `canonical` directive means `canonical=true`. Canonical
mode rewrites supported `_emphasis_` and `__strong__` spans as `*emphasis*` and
`**strong**`. It preserves protected or unsupported spans, including intraword
underscores, code spans, link destinations, raw HTML, and template content.

For `footnotes`, `wrap`, `format`, `true`, `yes`, and `1` format footnote
definitions. `preserve`, `none`, `false`, `no`, and `0` preserve them.
`--preserve-footnotes` preserves Markdown footnote definitions byte-for-byte.

## YAML settings

| Behavior | Control | Default | Effect |
| --- | --- | --- | --- |
| Structural line width | `--line-width <n>` | 80 | Expands block-renderable flow collections when they exceed the width. Over-width flow collections that cannot be rendered safely in block style may be preserved. Also bounds compact and layout-repair output. |
| Folded scalar prose width | `--prose-width <n>` | 72 | Wraps eligible folded YAML prose. |
| Block indentation | `--indent-width <n>` | 2 | Sets indentation for emitted block mappings and sequences. |
| Compact collections | `--compact`, `[format].compact`, or `fmt: compact` | Off | Collapses eligible block collections to flow style when they fit. |
| Aligned flow mappings | `fmt: table` or `fmt: compact table` | Off | Aligns a following sequence of compatible flow mappings. The compact form first collapses eligible block rows. |

All width and indentation values must be positive integers.

Canonical mode also permits limited rewrapping of short existing folded YAML
prose scalars when their value stays unchanged.

## Execution controls

| Control | Effect |
| --- | --- |
| `--diagnostics` | For YAML input, emits trace counters; it also emits notes for skipped or failing optional embedded formatters. |
| `--skip-embedded-formatters` | Disables external formatters while keeping Yamark's Markdown, YAML, front matter, and recursive Markdown formatting active. |
| `--config <path>` | Uses one explicit `yamark.toml` for all selected files instead of discovering one per file. |

### Embedded formatter controls

| Need | Control |
| --- | --- |
| Select a target | Use a recognized Markdown fence language or a [`fmt: <name>` directive](reference-directives.qmd#templates-and-embedded-formatters). |
| Register or replace a formatter | Add an [`[embedded.<name>]` table](reference-config.qmd#embedded) to `yamark.toml`. |
| Disable external formatters for one run | Pass `--skip-embedded-formatters`. |
| Explain an optional formatter skip | Pass `--diagnostics`. |

Write, check, diff, and stdin behavior is documented under
[Command line](cli-help.qmd#modes-output-and-status).

## Document Markdown options

Markdown front matter can set document-local options under
`editor_options.markdown`:

```markdown
---
editor_options:
  markdown:
    wrap: sentence:72
    canonical: true
    footnotes: preserve
---
This is __strong__. This is _emphasis_.
```

| Key | Accepted values |
| --- | --- |
| `wrap` | `none`, `paragraph`, `sentence`, `sentence:<n>`, or a positive integer column. |
| `canonical` | `true`, `false`, `yes`, `no`, `1`, or `0`. |
| `footnotes` | `wrap`, `format`, `preserve`, `none`, `true`, `false`, `yes`, `no`, `1`, or `0`. |

Unknown front-matter keys and unrecognized values are ignored; the
corresponding base setting remains active. Invalid `yamark.toml` keys or
directive values are errors instead.

These options apply after the front matter to the Markdown body, recursive
`markdown` or `md` fences, and Markdown-valued YAML scalars in nested YAML
regions.

Yamark also reads `editor.markdown` for compatibility, but only when
`editor_options.markdown` is absent. It does not merge the two tables. Git
filter subcommands use their own wrapping options and do not read document
options.

## Interaction rules

Markdown settings are applied in this order:

1. Command defaults and flags establish the base settings. `[format].wrap`
   replaces the default wrap setting, but an explicit `--wrap` replaces the
   config value.
2. Document front matter overrides the corresponding base Markdown settings
   for the document body and its nested Markdown regions.
3. Directives then override the corresponding settings within their scope.
4. `--preserve-footnotes` is the final exception: it preserves footnote
   definitions even when front matter or a directive requests formatting.

Other interactions:

- `--compact` enables compact mode even if `[format].compact` is `false`. A
  scoped `fmt: compact=false` directive can disable it for its target, region,
  or file.
- `editor_options.markdown` takes the place of the compatibility
  `editor.markdown` table when both are present; the tables are not merged.
- Git filter clean and smudge commands use their own Markdown wrapping options.
  They do not read document Markdown options.

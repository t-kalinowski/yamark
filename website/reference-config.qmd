---
title: Configuration
description: Discovery and the complete yamark.toml schema.
---

Yamark discovers `yamark.toml` from each formatted file's directory upward. It
uses only the nearest file; it does not merge ancestor configs.

Pass `--config path/to/yamark.toml` to use one explicit file for every selected
path instead. Unknown keys and invalid values are errors.

The top-level tables are `[format]`, `[template]`, `[embedded]`, and `[paths]`.

## `[format]`

```toml
[format]
wrap = "sentence:88"
compact = true
markdown_horizontal_rule = "***"
```

| Key | Type and values | Effect when present |
| --- | --- | --- |
| `wrap` | String: `none`, `paragraph`, `sentence`, `sentence:<n>`, or a positive integer. | Sets Markdown wrapping. An explicit `--wrap` value overrides it. |
| `table_widths` | String: `"fit"` or `"preserve"`. | Sizes Markdown table columns from their contents by default, or retains source widths. An explicit `--table-widths` overrides it; document settings and scoped directives override that base setting. |
| `compact` | Boolean. | Enables or disables eligible YAML block-to-flow compaction. `--compact` still enables compact mode when this is `false`. |
| `markdown_horizontal_rule` | String: `"---"` or `"***"`. | Chooses the marker Yamark emits for normalized Markdown horizontal rules. |

See [Formatting settings](reference-options.qmd) for built-in defaults and the
other surfaces that expose these behaviors.

## `[template]`

Template delimiters mark regions Yamark must preserve because rendering can
change the host language. The defaults are `{{ }}`, `{% %}`, `{# #}`, and
`<% %>`.

Yamark formats and wraps surrounding Markdown when a complete template pair
fits inside one inline code span, or a balanced braced expression such as
`{{ foo }}` fits on one source line. It does not insert wrap breaks inside
these expressions.

In paragraphs, multiline inline code keeps its existing line breaks. Its first
line can share a line with preceding prose, and following prose can fit beside
its last line. Lists, definition lists, blockquotes, and footnotes containing
multiline inline code remain unchanged.

Supported inline HTML regions remain opaque tokens: surrounding Markdown can
wrap before or after them, while their attributes and contents stay intact.
Yamark does not reflow text inside these regions as Markdown prose.

Within a Markdown block, a template that starts inside a code span and ends
outside it prevents reflow. So do multiline braced expressions. These checks
do not parse template regions across independent Markdown blocks.

Quarto is a first-class supported format. Other template systems receive
best-effort delimiter preservation: Yamark does not interpret tag names such
as Jinja's `raw` or Django/Twig's `verbatim`. Text between those tags is
formatted as ordinary Markdown.

Template-bearing blocks with apparent hard-break markers (two trailing spaces
or a trailing backslash) are preserved, including their trailing whitespace.

Values of `fig-alt` containing braces are not split for column wrapping.

Standalone shortcode tags such as `{{% notice %}}` and `{{% /notice %}}`
are preserved independently, including complete tags spanning multiple lines.
The tag ends at `%}}` or `>}}` outside quoted arguments. The text between tags
is ordinary Markdown; Yamark does not look for matching shortcode names. Use
Markdown code fences or explicit preservation directives for content that
must stay unchanged.

| Key | Type | Effect |
| --- | --- | --- |
| `add_delimiters` | Array of `{ open, close }` tables. | Appends to the delimiters active at this layer, after `replace_delimiters` if both keys are present. |
| `replace_delimiters` | Array of `{ open, close }` tables. | Replaces the delimiters active at this layer before any additions. |

Add a delimiter pair:

```toml
[template]
add_delimiters = [
  { open = "<<", close = ">>" }
]
```

Replace the defaults:

```toml
[template]
replace_delimiters = [
  { open = "[[", close = "]]" }
]
```

Every entry must contain non-empty `open` and `close` strings.

## `[embedded]`

Each child table maps a directive or fence name to a formatter that reads stdin
and writes stdout. It accepts exactly one key: the required `formatter` key.

```toml
[embedded.python]
formatter = "ruff"

[embedded.r]
formatter = "air"

[embedded.sql]
formatter = { command = ["sqlfmt", "--filename", "{path}"], path_suffix = ".sql" }
```

`formatter` accepts either a built-in shorthand or a custom formatter table.
Built-in shorthands are:

- `ruff`
- `air`
- `mdformat`
- `prettier-json`, `prettier-jsonc`, and `prettier-json5`
- `prettier-graphql`
- `prettier-css`, `prettier-scss`, `prettier-less`, and `prettier-postcss`
- `prettier-html`
- `prettier-js`, `prettier-jsx`, `prettier-ts`, and `prettier-tsx`

A custom formatter table requires both `command` and `path_suffix` and accepts
no other keys:

| Key | Requirement |
| --- | --- |
| `command` | A non-empty argv array of strings. It is not a shell command. At least one complete argv item must be `{path}`. |
| `path_suffix` | A non-empty suffix appended to the synthetic path passed to the formatter. |

`{path}` cannot be embedded inside another argv item.

Built-in formatters are optional. A configured formatter is also optional when
its command's first argv item is exactly `ruff`, `air`, `mdformat`, or
`prettier`. For either kind, a missing executable or nonzero exit preserves the
target and emits a note only with `--diagnostics`. Any other configured
formatter treats either condition as an error. A successful process that
writes to stderr is always an error.

Embedded formatter names must be non-empty and trimmed. `skip`, `skip file`,
`off`, `on`, and `table` are reserved directive names.

## `[paths]`

Path keys are relative to the directory containing `yamark.toml`. They must not
be empty or absolute and must not contain `..`.

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

| Table | Effect |
| --- | --- |
| `paths.<path>.template` | Adds or replaces generic template delimiters for matching files. |
| `paths.<path>.embedded_markdown.template` | Adds or replaces delimiters only for Markdown embedded in source strings or comments. |

Both template tables accept `add_delimiters` and `replace_delimiters` with the
same entry schema as top-level `[template]`. Yamark starts with the top-level
delimiters, then applies matching path layers from the shallowest path to the
deepest. Each `replace_delimiters` discards delimiters accumulated by earlier
layers before that layer's `add_delimiters` are appended.

Configured embedded-Markdown delimiters do not apply inside Python f-strings.
Those strings preserve Python `{...}` expressions instead.

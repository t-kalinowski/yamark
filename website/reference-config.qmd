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
| `compact` | Boolean. | Enables or disables eligible YAML block-to-flow compaction. `--compact` still enables compact mode when this is `false`. |
| `markdown_horizontal_rule` | String: `"---"` or `"***"`. | Chooses the marker Yamark emits for normalized Markdown horizontal rules. |

See [Formatting settings](reference-options.qmd) for built-in defaults and the
other surfaces that expose these behaviors.

## `[template]`

Template delimiters mark regions Yamark must preserve because rendering can
change the host language. The defaults are `{{ }}`, `{% %}`, `{# #}`, and
`<% %>`.

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

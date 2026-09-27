Yamark 0.4.0 adds Markdown table-width controls, formats prose around template expressions, and improves preservation of inline literals and shortcodes.

## Command line

- Markdown pipe and Pandoc tables now fit their columns to the contents by
  default while retaining alignment and multiline cell breaks. Use
  `--table-widths preserve`, `[format].table_widths`, front matter, or a scoped
  `fmt: table-widths=preserve` directive to retain authored widths. Headerless
  Pandoc tables and tables containing tabs are handled more consistently.
- Complete single-line template expressions retain their contents while
  surrounding prose wraps in paragraphs and supported containers. Recognized
  inline code and math retain their whitespace and line endings. Standalone
  `{{< ... >}}` and `{{% ... %}}` shortcode tokens, including multiline tokens
  with quoted arguments, stay intact while Markdown between them formats.
- `fmt: skip`, `fmt: off`/`fmt: on`, and `fmt: skip file` retain skipped Markdown
  bytes, including trailing spaces and line endings, in supported nested
  Markdown.
- YAML formatting removes indentation-only spaces from nontrailing blank lines
  inside block scalars. Safe flow strings such as `:workspace` and `?query` no
  longer gain unnecessary quotes. `--compact` avoids braces on single-pair
  root mappings and one-line mappings inside sequences when they would not
  save a line. Fix repeated formatting of flow values such as `[it's quoted]`
  after another mapping entry.
- Fix a panic when formatting headings with non-ASCII text and attached
  attributes, such as `# Café{#id}`.
- Formatting avoids repeated work in long Markdown headings, list blank lines,
  YAML prose, and YAML streams. `--check` and `--diff` retain fewer intermediate
  file buffers and diffs.

## VS Code and Positron

- JSONL and NDJSON read-only YAML views now refresh when the source file changes on disk. The initial view includes unsaved editor changes. If an appended record is incomplete, the view keeps the last successful output until the source changes again.

## Rust API and distribution

- `SourceSpan`, `Document`, and their stored node and emission types no longer take source lifetime parameters. Text accessors borrow the supplied source buffer.
- Building from source requires Rust 1.98.1 or newer.
- The release includes 64-bit ARM Linux (`aarch64-unknown-linux-gnu`) archives and wheels.

**Full changelog:** https://github.com/t-kalinowski/yamark/compare/v0.3.0...v0.4.0

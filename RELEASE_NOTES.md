<!-- Draft notes for the next release here as user-facing changes land. See RELEASE.md. -->

- Markdown wrapping treats template spans as opaque words, including in lists and blockquotes. `{{` words end at the first literal `}}`, including across lines; quotes and nested braces do not affect that boundary. The default literal pair `{{< raw >}}` / `{{< /raw >}}` preserves its whole payload. Longer opening delimiters take precedence; custom pairs can use `literal = true` for the same exact matching. Template contents stay exact, while template-only lines can join surrounding prose or result from wrapping. Use `fmt: skip`, `fmt: off`/`fmt: on`, or `fmt: skip file` to preserve layout.
- Markdown tables fit their contents by default while retaining column alignment. Use `table-widths=preserve` in a scoped directive or document setting, `[format].table_widths` in `yamark.toml`, or `--table-widths preserve` to retain intentional source widths. Fitting can change Pandoc's rendered column proportions. Multiline cells retain their line breaks, and headerless tables end at their closing border.
- Building from source now requires Rust 1.98.1 or newer.
- Fix a panic when formatting headings with non-ASCII text and attached attributes, such as `# Café{#id}`.
- Reduce repeated work when formatting long Markdown headings, unmatched brackets, list blank lines, YAML prose, and YAML streams, and when generating diffs.
- `--check` releases formatted file contents inside each worker, and `--diff` retains completed diffs instead of the original and formatted files.
- Fix repeated formatting of flow values such as `[it's quoted]` after another mapping entry.
- The Rust API no longer exposes `MarkdownNodeKind::Shortcode` or `contains_markdown_template_span`; template detection uses `contains_template_span`. `TemplateDelimiter` adds a `literal` field for exact closing-string matching, and its `open` and `close` fields use `Cow<'static, str>` so default strings can be borrowed.
- In the Rust API, `SourceSpan`, `Document`, and their stored node and emission types no longer take source lifetime parameters. Text accessors borrow the supplied source buffer. Spans retain their compact representation and bounds checks.
- YAML strings such as `:workspace` and `?query` no longer gain unnecessary quotes when formatted in flow collections.
- `--compact` keeps single-pair root mappings and one-line block mappings in sequences free of unnecessary braces.
- Releases now include 64-bit ARM Linux (`aarch64-unknown-linux-gnu`) archives and wheels.

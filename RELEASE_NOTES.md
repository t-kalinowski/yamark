<!-- Draft notes for the next release here as user-facing changes land. See RELEASE.md. -->

- Markdown prose now wraps around complete template pairs inside inline code and balanced braced expressions on one source line, such as `{{ foo }}`, while preserving their contents and keeping each expression together. Blocks mixing templates and HTML, or containing a bare template expression on its own line, remain unchanged.
- Standalone shortcode tags remain unchanged while the Markdown between them is formatted normally. Yamark no longer looks for a matching closing shortcode name; use Markdown code fences or explicit preservation directives for content that must stay unchanged.
- Markdown tables fit their contents by default while retaining column alignment. Use `table-widths=preserve` in a scoped directive or document setting, `[format].table_widths` in `yamark.toml`, or `--table-widths preserve` to retain intentional source widths. Fitting can change Pandoc's rendered column proportions. Multiline cells retain their line breaks, and headerless tables end at their closing border.
- Building from source now requires Rust 1.98.1 or newer.
- Fix a panic when formatting headings with non-ASCII text and attached attributes, such as `# Café{#id}`.
- Reduce repeated work when formatting long Markdown headings, unmatched brackets, list blank lines, YAML prose, and YAML streams, and when generating diffs.
- `--check` releases formatted file contents inside each worker, and `--diff` retains completed diffs instead of the original and formatted files.
- Fix repeated formatting of flow values such as `[it's quoted]` after another mapping entry.
- In the Rust API, `SourceSpan`, `Document`, and their stored node and emission types no longer take source lifetime parameters. Text accessors borrow the supplied source buffer. Spans retain their compact representation and bounds checks.
- YAML strings such as `:workspace` and `?query` no longer gain unnecessary quotes when formatted in flow collections.
- `--compact` keeps single-pair root mappings and one-line block mappings in sequences free of unnecessary braces.

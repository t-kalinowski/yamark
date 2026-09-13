<!-- Draft notes for the next release here as user-facing changes land. See RELEASE.md. -->

- Pandoc tables keep their column alignment and widths when formatted. Multiline cells retain their line breaks, and headerless tables end at their closing border.
- Building from source now requires Rust 1.98.1 or newer.
- YAML strings such as `:workspace` and `?query` no longer gain unnecessary quotes when formatted in flow collections.
- `--compact` keeps single-pair root mappings and one-line block mappings in sequences free of unnecessary braces.

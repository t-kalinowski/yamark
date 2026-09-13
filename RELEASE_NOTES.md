<!-- Draft notes for the next release here as user-facing changes land. See RELEASE.md. -->

- Building from source now requires Rust 1.98.1 or newer.
- In the Rust API, `SourceSpan`, `Document`, and their stored node and emission types no longer take source lifetime parameters. Text accessors borrow the supplied source buffer. Spans retain their compact representation and bounds checks.
- YAML strings such as `:workspace` and `?query` no longer gain unnecessary quotes when formatted in flow collections.
- `--compact` keeps single-pair root mappings and one-line block mappings in sequences free of unnecessary braces.

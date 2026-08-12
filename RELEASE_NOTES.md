Yamark 0.3.0 adds JSON-to-YAML projection, read-only editor previews, and targeted improvements to YAML formatting.

## Command line

- Adds `yamark to-yaml --stdin-file-path PATH` for converting JSON, JSONC, JSON5, JSONL, and NDJSON from stdin to formatted YAML on stdout. JSONC and JSON5 comments become YAML comments, object member order and duplicate names are preserved, and each JSONL or NDJSON record becomes a separate YAML document. The path selects the input grammar but is never read or written. This is conversion rather than source formatting: `yamark format` continues to skip JSON-family files.
- YAML streams made from two or more unmarked, one-line flow mappings now receive a `---` marker before every document. Normal width settings still determine whether each mapping remains compact or expands.
- Multiline quoted YAML strings now become literal blocks in more supported contexts, including sequence entries containing colons and values inside flow collections that expand.
- Expanded flow collections now wrap eligible long prose as folded blocks and use compact `- key: value` layout for mapping entries in sequences.

No released command-line option was removed.

## VS Code and Positron

- Adds `Yamark: Preview Format Document` for Markdown, R Markdown, Quarto, YAML, Python, and R. It runs the same formatting pipeline as Format Document, including a configured next formatter, and opens the result without applying it.
- Adds `Yamark: View JSON as YAML` for JSON, JSONC, JSON5, JSONL, and NDJSON. It uses `yamark to-yaml` and keeps JSON-family files excluded from Format Document and format-on-save.
- Both commands include unsaved buffer changes and open a read-only snapshot without changing the source or creating a temporary file. Run the command again to refresh the preview.

## Documentation

- Reorganizes the website reference into focused pages for supported files and syntax, formatting settings, configuration, directives, and command-line behavior. The site now includes reference search and clearer examples of YAML streams, JSON projection, Markdown, and embedded source formatting.

**Full changelog:** https://github.com/t-kalinowski/yamark/compare/v0.2.1...v0.3.0

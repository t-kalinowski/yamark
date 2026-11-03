Yamark 0.5.0 adds wrapping for multiline YAML prose and fixes Markdown wrapping, configuration discovery, Git filters, and editor formatting.

## Command line

### YAML formatting

- Yamark can now rewrap long YAML strings written across multiple lines without `|` or `>` block syntax, whether unquoted, single-quoted, or double-quoted. It can also format more documents containing multiline quoted strings while preserving comments and string values.
- Fixed a case where formatting YAML twice could produce different layouts. It affected lists and mappings containing escaped strings that look like numbers.
- Ordinary YAML scalars remain plain or quoted when conversion to block style would absorb a following indented comment into the value. Explicit Markdown targets use deeper content indentation when needed to keep the following comment outside the value.
- Removing YAML parent indentation now preserves quoted continuation indentation, avoiding a panic on low-indented continuations and keeping quoted document-marker text inside the value.

### Markdown formatting

- Sentence and column wrapping avoid turning literal punctuation into lists, headings, or other blocks, with fixes for escaped punctuation, emphasis, inline math, and HTML.
- Wrapping preserves empty and angle-bracket link destinations and keeps split links and images valid.
- Formatting preserves outer list paragraphs after nested lists and retains heading and raw-block boundaries in lists and block quotes.
- Embedded formatter output retains complete CRLF line endings when a formatter removes the final newline character.

### Configuration and Git filters

- Yamark now finds `yamark.toml` above the current directory when you pass a relative file or directory path, including a relative `--stdin-file-path`.
- `--diff` now produces patches that apply correctly when changes are close together.
- Git filter `adopt`, `join`, and `check` now handle filenames that Git prints with quotes or escape sequences, including non-UTF-8 filenames on Unix.
- Git filter `adopt` and `join` respect Git's line-ending settings when writing working-tree files.
- Git filter `check --repo PATH` now uses the selected repository's formatting settings, including settings in its subdirectories.

## VS Code and Positron

- If you edit a document while formatting is running, the editor discards the outdated result.
- Selection formatting applies to the text selected when formatting started, even if you move the selection before it finishes.

**Full changelog:** <https://github.com/t-kalinowski/yamark/compare/v0.4.0...v0.5.0>

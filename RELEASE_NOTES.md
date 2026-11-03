<!-- Draft notes for the next release here as user-facing changes land. See RELEASE.md. -->

- **YAML formatting**
  - Yamark can now rewrap long YAML strings written across multiple lines without `|` or `>` block syntax, whether unquoted, single-quoted, or double-quoted. It can also format more documents containing multiline quoted strings while preserving comments and string values.
  - Fixed a case where formatting YAML twice could produce different layouts. It affected lists and mappings containing escaped strings that look like numbers.
  - Plain and quoted YAML scalars retain their spelling when conversion to block style would absorb a following indented comment into the value.

- **Command line**
  - Yamark now finds `yamark.toml` above the current directory when you pass a relative file or directory path, including a relative `--stdin-file-path`.
  - `--diff` now produces patches that apply correctly when changes are close together.

- **Git filters**
  - `adopt`, `join`, and `check` now handle filenames that Git prints with quotes or escape sequences, including non-UTF-8 filenames on Unix.
  - `check --repo PATH` now uses the selected repository's formatting settings, including settings in its subdirectories.

- **Editor integration**
  - If you edit a document while formatting is running, the editor discards the outdated result.
  - Selection formatting applies to the text selected when formatting started, even if you move the selection before it finishes.

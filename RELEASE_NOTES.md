<!-- Draft notes for the next release here as user-facing changes land. See RELEASE.md. -->

- Relative file, directory, and stdin paths now discover `yamark.toml` in ancestor directories above the working directory.
- Diffs merge overlapping context so nearby changes produce applicable patches.
- Git filter adoption, joining, and checking handle filenames that Git would quote in line-oriented output, including non-UTF-8 names on Unix.
- Git filter verification with `--repo` reads formatting configuration from the selected repository, including nested configuration files.
- YAML flow layout accounts for required quotes around escaped numeric-looking strings, keeping the layout stable on a second formatting pass.
- YAML prose already split across plain or quoted continuation lines can be wrapped into folded blocks. Supported multiline quoted values allow the surrounding document to be formatted while preserving comments and scalar values.

## Editor integration

Formatting discards pending results when the document changes. Selection formatting retains the range selected when the command started, so moving the selection while formatting does not replace a different range.

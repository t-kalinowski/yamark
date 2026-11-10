<!-- Draft notes for the next release here as user-facing changes land. See RELEASE.md. -->

## Command line

### Markdown formatting

- Treat `No.` as an abbreviation only when the next whitespace-separated source token starts with an ASCII digit, as in `No. 5`. Otherwise, sentence wrapping starts a new line after it, and column wrapping may break there.

<!-- Draft notes for the next release here as user-facing changes land. See RELEASE.md. -->

## Command line

### Markdown formatting

- Treat `c.`, `d.`, `f.`, `ill.`, `n.`, `No.`, `p.`, `pt.`, and `sec.` as abbreviations only when the next whitespace-separated source token starts with an ASCII digit, as in `p. 5` or `No. 5`. Otherwise, sentence wrapping starts a new line after them, and column wrapping may break there.

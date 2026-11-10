<!-- Draft notes for the next release here as user-facing changes land. See RELEASE.md. -->

## Command line

### Markdown formatting

- Treat `c.`, `d.`, `f.`, `ill.`, `n.`, `No.`, `p.`, `pt.`, and `sec.` as abbreviations only when the next whitespace-separated source token starts with an ASCII digit, as in `p. 5` or `No. 5`. Otherwise, sentence wrapping starts a new line after them, and column wrapping may break there.
- Treat `a.k.a.`, `approx.`, `E.g.`, `eg.`, `Eq.`, `Eqs.`, `Fig.`, `Figs.`, `I.e.`, `ie.`, `resp.`, and `w.r.t.` as abbreviations: they stay on the same line as the following word and do not end a sentence for wrapping. Treat `etc.` and `i.i.d.` the same way when the next whitespace-separated source token starts with a lowercase letter.
- Do not end a sentence at an abbreviation followed by closing quotation marks or brackets, as in `you could (e.g.) have a policy`.

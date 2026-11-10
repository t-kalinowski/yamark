<!-- Draft notes for the next release here as user-facing changes land. See RELEASE.md. -->

## Command line

### Markdown formatting

- Treat `c.`, `d.`, `f.`, `ill.`, `n.`, `No.`, `p.`, `pt.`, and `sec.` as abbreviations only when the next whitespace-separated source token starts with an ASCII digit, as in `p. 5` or `No. 5`. Otherwise, sentence wrapping starts a new line after them, and column wrapping may break there.
- Treat `a.k.a.`, `approx.`, `E.g.`, `eg.`, `Eq.`, `Eqs.`, `Fig.`, `Figs.`, `I.e.`, `ie.`, `resp.`, and `w.r.t.` as abbreviations: they stay on the same line as the following word and do not end a sentence for wrapping. Treat `etc.` and `i.i.d.` the same way when the next whitespace-separated source token starts with a lowercase letter.
- Do not end a sentence at an abbreviation followed by closing quotation marks or brackets, as in `you could (e.g.) have a policy`.
- End a sentence at `&#46;`, the decimal character reference for a period, so that an author can mark a sentence end after an abbreviation, as in `We follow Smith et al&#46; Their method is faster.` It renders as a period.
- Add `fmt: abbreviations.add` and `fmt: abbreviations.remove` directives, which make the listed words abbreviations or ordinary words for wrapping, as in `<!-- fmt: abbreviations.add "Tab." "Thm." -->`. They accept `scope=next`, `scope=from-here`, and `scope=file`.
- Add a `[format.abbreviations]` table to `yamark.toml`. Its `add` and `remove` arrays make the same corrections as the directives for every file the configuration applies to; a directive that names the same word overrides them.

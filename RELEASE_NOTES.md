Yamark 0.6.1 tunes where Markdown sentence wrapping ends a sentence in technical prose. It also adds ways to correct a boundary: the `&#46;` sentence-end marker, `fmt: abbreviations.add` and `fmt: abbreviations.remove` directives, and a `[format.abbreviations]` table in `yamark.toml`.

## Command line

### Markdown formatting

- Treat `c.`, `d.`, `f.`, `ill.`, `n.`, `No.`, `p.`, `pt.`, and `sec.` as abbreviations only when the next whitespace-separated source token starts with an ASCII digit, as in `p. 5` or `No. 5`. Otherwise, sentence wrapping starts a new line after them, and column wrapping may break there.
- Treat `a.k.a.`, `approx.`, `E.g.`, `eg.`, `Eq.`, `Eqs.`, `Fig.`, `Figs.`, `I.e.`, `ie.`, `resp.`, and `w.r.t.` as abbreviations: they stay on the same line as the following word and do not end a sentence for wrapping. Treat `etc.` and `i.i.d.` the same way when the next whitespace-separated source token starts with a lowercase letter.
- Treat `al.` as an abbreviation unless the next whitespace-separated source token starts with an uppercase letter, so `We follow Smith et al. Their method is faster.` becomes two sentences for wrapping while `et al. (2020)` and `et al. showed` stay together.
- Do not end a sentence at an abbreviation followed by closing quotation marks or brackets, as in `you could (e.g.) have a policy`.
- End a sentence at `&#46;`, the decimal character reference for a period, so that an author can mark a sentence end after an abbreviation, as in `We work with Acme Inc&#46; Their API is faster.` It renders as a period.
- Add `fmt: abbreviations.add` and `fmt: abbreviations.remove` directives, which make the listed words abbreviations or ordinary words for wrapping, as in `<!-- fmt: abbreviations.add "Tab." "Thm." -->`. They accept `scope=next`, `scope=from-here`, and `scope=file`.
- Add a `[format.abbreviations]` table to `yamark.toml`. Its `add` and `remove` arrays make the same corrections as the directives for every file the configuration applies to; a directive that names the same word overrides them.

**Full changelog:** <https://github.com/t-kalinowski/yamark/compare/v0.6.0...v0.6.1>

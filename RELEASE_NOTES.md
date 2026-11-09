<!-- Draft notes for the next release here as user-facing changes land. See RELEASE.md. -->

## Command line

### Markdown formatting

- Keep thematic-break markers and later setext-like lines inside continuing paragraphs, following Pandoc's single-line setext heading rules.
- Separate headings from following horizontal rules so normalizing a rule cannot turn the heading into a different setext heading.
- Keep list item ownership through fenced child blocks and nested lists. Paragraphs after a fence and multiline nested items beneath wide parent markers can now be formatted without losing their container context.
- Keep fenced divs and grid tables in list items intact. A div remains one child block across blank lines, and a list is preserved when shortening its marker would move a div off the item's content column.
- Preserve unsupported list continuations containing non-breaking spaces without slicing through a UTF-8 character and panicking.
- Treat U+FEFF as a byte order mark only at the start of the file, as Pandoc does. A first line holding only a byte order mark and spaces is now blank, which fixes a panic when `---` or `===` follows it, keeps `---` after a byte-order-mark-only line as a thematic break, and no longer adds a hard line break before the next line. A U+FEFF later in the file, including at the start of blockquote or footnote content, is text, so a setext heading containing it keeps its content.
- With the optional `format-trace` feature, report retained preservation and unavailable layout plans as skipped, and omit descendant decisions for containers that are explicitly preserved.

### YAML formatting

- A file-scoped `fmt: template.delimiters` directive placed after the YAML it applies to now preserves template-containing values, as it does when placed before them. Previously, a late directive could split a flow collection such as `[<< a,b >>]` at the template's internal comma. This also applies to YAML inside Markdown fences.

### Embedded formatters

- An embedded formatter that exits successfully may now close stdin before reading all of its input. Yamark previously reported a broken-pipe error for large inputs. The formatter's output still receives normal validation, and other stdin write errors still fail.

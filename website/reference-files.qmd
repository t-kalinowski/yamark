---
title: Supported files and syntax
description: Files, nested regions, syntax behavior, layout repair, and failure boundaries.
---

Use this page to determine what Yamark selects and what it does with a specific
construct. [Formatting settings](reference-options.qmd) describes how to change
the output.

## Files and regions

| Input | What Yamark formats | Selection |
| --- | --- | --- |
| `.yaml` and `.yml` | The whole YAML stream. | Automatic for path-aware formatting. |
| `.md`, `.qmd`, `.Rmd`, and `.rmd` | Markdown, YAML front matter, and recognized fenced content. | Automatic, except constructs described as preserved below. |
| `.py`, `.R`, and `.r` | `#|` hashpipe YAML blocks, marked Markdown comments or strings, and marked external-formatter targets. | Hashpipe blocks are automatic; other targets require a `fmt:` directive. Surrounding source code is unchanged. |
| Markdown fences named `yaml`, `yml`, `markdown`, or `md` | YAML, or recursively formatted Markdown. | Automatic unless the fence is locally skipped. |
| Markdown source fences | Python, R, JSON, GraphQL, CSS, HTML, JavaScript, TypeScript, and related aliases. | Delegated when a matching embedded formatter is available. |
| YAML scalars tagged `!markdown` or `!md` | The scalar value as Markdown. | Explicit tag. |
| YAML scalars following `# fmt: markdown` | The scalar value as Markdown. | Explicit directive. |
| YAML literal scalars following `# fmt: <name>` or `# fmt: embedded <name>` | The scalar value through a built-in or configured embedded formatter. | Explicit directive. |

Extension matching is ASCII case-insensitive. In path mode, unsupported
extensions are counted as skipped and do not make the run fail. An unsupported
`--stdin-file-path` is an error because stdin mode requires file-aware behavior.

Directory traversal skips hidden paths and respects `.gitignore`, `.ignore`,
and global Git ignore files. Passing a hidden path explicitly selects it.

## Markdown

| Construct | Result | Details |
| --- | --- | --- |
| Paragraphs | Formats | Wraps by column, paragraph, sentence, or not at all, according to the active Markdown settings. |
| ATX headings | Formats | Normalizes spacing. Preserves and, when safe, compacts heading attributes. |
| Lists and task lists | Formats | Reflows list paragraphs with container indentation accounted for. Preserves GFM task markers. |
| Blockquotes | Formats when supported | Wraps supported blockquote shapes recursively. |
| Footnote blocks | Formats or preserves | Wraps definitions by default. A preserve setting keeps definitions byte-for-byte. |
| Reference links | Formats around | Treats reference-style links as atomic wrapping tokens. Preserves reference definitions without relocation. |
| Nested image links | Formats when safe | Normalizes and wraps simple nested image labels and long destinations. |
| Pandoc citations | Formats around | Protects citation spans such as `[@key]` and `[-@key]` while wrapping. |
| Quarto divs | Formats when supported | Formats supported fenced div bodies recursively. |
| YAML fences | Formats | Formats fenced `yaml` and `yml` blocks as YAML unless locally skipped. |
| Markdown fences | Formats | Formats fenced `markdown` and `md` blocks recursively. |
| Source fences | Delegates | Can run embedded formatters for Python, R, JSON, GraphQL, CSS, HTML, JavaScript, TypeScript, and related aliases. |
| Long Quarto fence openings | Formats when safe | Promotes simple comma-separated options such as `echo=FALSE` to `#| echo: false` lines for a supported fence when wrapping is not `none` and the opening exceeds the configured column, or 72 in `paragraph` and `sentence` modes. For a delegated language, the embedded formatter must succeed; a missing, failed, or disabled formatter preserves the original opening. |
| Quarto source fences with `#| fmt: skip` | Preserves | Leaves the fenced chunk unchanged when the directive appears anywhere in the initial consecutive `#|` option block. |
| Document Markdown options | Reads | Applies `editor_options.markdown.{wrap,canonical,footnotes}` to the body and nested Markdown that follows. |
| GFM pipe tables | Formats | Aligns supported tables by display width. Git clean/smudge filters use compact pipe-table output. |
| Pandoc tables | Formats when supported | Normalizes supported simple, grid, and multiline tables. |
| Definition lists | Formats when supported | Normalizes marker spacing and wraps definitions. |
| Display math | Preserves | Keeps own-line `$$` display math blocks byte-for-byte. |
| Raw HTML, TeX, Hugo shortcodes, table captions, and line blocks | Preserves | Leaves the construct unchanged when Yamark does not have a safe rewrite. |
| Template spans | Preserves | Recognizes default and configured template delimiters. |

For Quarto header promotion, a supported fence is a YAML or Markdown fence, a
language with a built-in or configured formatter, or one of the opaque
languages Yamark preserves safely: `bash`, `sh`, `shell`, `zsh`, `ojs`, `text`,
`console`, `rust`, `toml`, `lua`, `mermaid`, `ini`, `julia`, `sql`, `java`, `c`,
`tex`, `latex`, `output`, `powershell`, and `cmd`.

## YAML

| Construct | Result | Details |
| --- | --- | --- |
| Block mappings and sequences | Formats | Normalizes indentation and spacing. Does not reorder keys. |
| Flow mappings and sequences | Formats | Normalizes spacing. Expands multiline or over-width collections to block style when safe. |
| Compact collections | Formats when enabled | Collapses eligible block collections to flow style with `--compact`, `[format].compact`, or `fmt: compact`. |
| Scalar folding | Formats when safe | Can emit and rewrap folded prose scalars at the active prose width. |
| Literal scalars | Preserves by default | Changes a literal scalar only when it is explicitly marked as Markdown or an embedded formatter target. |
| Quoted scalars with hard newlines | Formats when safe | Can emit literal block style when the YAML value stays unchanged. |
| Unsafe plain scalars | Quotes | Quotes scalars whose meaning would change in block or flow context. |
| Core booleans and nulls | Formats | Normalizes plain values to YAML 1.2 spellings such as `true`, `false`, and `null`. Normalizes explicit core tags when safe. |
| Tags and anchors | Preserves | Preserves custom tags, anchors, aliases, and tag/anchor order. May remove a redundant core collection tag when syntax already implies the type. |
| Comments and directives | Preserves and reads | Preserves comments around supported nodes. Reads `fmt:` comments as local controls. |
| Duplicate keys | Formats around | Does not validate or reorder duplicate keys. May format their values. |
| UTF-8 BOM and line endings | Preserves | Preserves a UTF-8 BOM and the dominant CRLF or CR line ending. Rejects UTF-16 BOM input. |
| Tab indentation | Preserves or rejects | Preserves tab-indented YAML when no active target requires interpreting it. Rejects targeted formatting that would need to parse the unsupported indentation. |

### JSON Lines (JSONL) as YAML streams

Path-aware formatting recognizes a `.yaml` or `.yml` stream made of two or more
unmarked, one-line YAML flow-mapping roots, one per physical line with no
separate trivia lines. It inserts the `---` document-start marker before each
record after the first. The first record has no leading marker.

JSON objects are the common case, but the trigger is based on YAML shape and is
not restricted to strict JSON objects.

Normal `--line-width` rules still determine whether each record and its nested
collections stay in flow style or expand to block style. A single flow-mapping
root or a stream containing any non-flow-mapping root follows the normal YAML
path instead.

See the generated [JSON Lines example](examples.qmd#json-lines-as-a-yaml-stream).

## Python and R source files

| Construct | Result | Details |
| --- | --- | --- |
| Consecutive own-line `#|` comments | Formats | Parses the comment body as YAML and emits it with the original hashpipe prefix. |
| Marked Markdown comments and strings | Formats | Formats supported own-line comment blocks and supported multiline string literals, accounting for their prefix or indentation. |
| Marked external source strings | Delegates | Sends a supported multiline string literal to the named built-in or configured formatter and re-indents the result. |
| Surrounding Python or R | Preserves | Does not format source outside recognized or explicitly marked targets. |

Every non-empty line in a comment target must use the same comment prefix;
empty comment lines may omit its trailing space. Python string targets use
triple quotes with no prefix, or an `r`, `f`, `rf`, or `fr` prefix; the closing
delimiter must be on a later line. R targets use a multiline standard or raw
string. Markdown formatting rejects backslashes in non-raw Python and R strings
because rewriting them could change the host-language value.

The generated [hashpipe YAML example](examples.qmd#hashpipe-yaml-in-source-files)
shows the automatic source-file transformation.

## Embedded formatter dispatch

External formatters run only for explicit targets or recognized Markdown fence
languages. Built-in aliases include `ruff`, `air`, `mdformat`, and the
`prettier` family.

| Target name or fence language | Built-in formatter |
| --- | --- |
| `python`, `ruff` | Ruff |
| `r`, `air` | Air |
| `mdformat` | mdformat |
| `json`, `prettier-json` | Prettier with a `.json` path |
| `jsonc`, `prettier-jsonc` | Prettier with a `.jsonc` path |
| `json5`, `prettier-json5` | Prettier with a `.json5` path |
| `graphql`, `gql`, `graphqls`, `prettier-graphql` | Prettier with a `.graphql` path |
| `css`, `prettier-css` | Prettier with a `.css` path |
| `scss`, `prettier-scss` | Prettier with a `.scss` path |
| `less`, `prettier-less` | Prettier with a `.less` path |
| `postcss`, `pcss`, `prettier-postcss` | Prettier with a `.pcss` path |
| `html`, `prettier-html` | Prettier with a `.html` path |
| `js`, `javascript`, `prettier-js` | Prettier with a `.js` path |
| `jsx`, `prettier-jsx` | Prettier with a `.jsx` path |
| `ts`, `typescript`, `prettier-ts` | Prettier with a `.ts` path |
| `tsx`, `prettier-tsx` | Prettier with a `.tsx` path |

Markdown fences named `markdown` or `md` are formatted recursively instead of
using mdformat. An [`[embedded.<name>]`](reference-config.qmd#embedded) entry
replaces the built-in mapping for that exact name.

The default aliases are optional. A configured formatter is also optional when
its command's first argv item is exactly `ruff`, `air`, `mdformat`, or
`prettier`. A missing executable or nonzero exit preserves the target for
either kind. Yamark reports the reason as a note only with `--diagnostics`.

A successful process that writes to stderr is an error. Any other configured
formatter is strict: a missing executable or nonzero exit is an error. These
formatter errors happen before Yamark writes the formatted file. See the
[`[embedded]` schema](reference-config.qmd#embedded) and
[`--skip-embedded-formatters`](reference-options.qmd#execution-controls).

## Layout repair

Yamark recognizes two narrow layout cues. Only the unmatched opener is a repair
for ill-formed input:

- An unmatched `[` or `{` where a block sequence or mapping begins as a mapping
  value or sequence item asks Yamark to collapse that one collection to flow
  style.
- A physical newline inside a complete flow collection is valid multiline YAML.
  It asks Yamark to expand that collection to block style. Compact mode
  suppresses this multiline intent.

The opener may be on the value line or on the following line after an empty
mapping value header. Adjacent forms such as `tags:[` and `tags:{` are not
layout hints.

When the requested flow form fits the available `--line-width`, Yamark removes
the unmatched opener and emits the collection in flow style. Otherwise it
preserves the hinted collection as typed. It does not fail solely because the
requested flow form is too wide, and surrounding YAML may still be formatted.
The [layout examples](examples.qmd#collapse-to-flow-by-typing-a-bracket) show
each accepted gesture before and after formatting.

## Failure behavior

Yamark is a formatter, not a general validator. It formats supported regions
and leaves unsupported regions unchanged when they are not explicitly
targeted and it has no safe rewrite.

Yamark parses YAML before formatting. For Markdown and Markdown-valued YAML
scalars, it emits only transformations supported by its parser. An embedded
formatter can change only a recognized fence or an explicitly marked source or
YAML target.

An explicit directive instead requires a supported target. A missing or
unsupported target or an unknown formatter name is an error for that file.
Formatter execution follows the optional and strict failure rules above.

Input-decoding, parsing, formatting, file-read, and external-formatter errors
happen before Yamark writes that file, so these errors leave its original
content unchanged. A write error is reported, but writes are direct rather than
atomic; Yamark does not promise rollback after a write begins.

A multi-file write run is not transactional. Other files can be written even
when one input fails.
